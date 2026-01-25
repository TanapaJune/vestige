import Database from 'better-sqlite3';
import { nanoid } from 'nanoid';
import type { GraphEdge } from '../core/types.js';

// ============================================================================
// EDGE TYPES
// ============================================================================

export type EdgeType =
  | 'relates_to'
  | 'contradicts'
  | 'supports'
  | 'similar_to'
  | 'part_of'
  | 'caused_by'
  | 'mentions'
  | 'derived_from'
  | 'references'
  | 'follows'
  | 'person_mentioned'
  | 'concept_instance';

// ============================================================================
// INPUT TYPES
// ============================================================================

export interface GraphEdgeInput {
  fromId: string;
  toId: string;
  edgeType: EdgeType;
  weight?: number;
  metadata?: Record<string, unknown>;
}

// ============================================================================
// TRANSITIVE PATH TYPE
// ============================================================================

export interface TransitivePath {
  path: string[];
  totalWeight: number;
}

// ============================================================================
// RWLOCK - Read-Write Lock for concurrent access control
// ============================================================================

/**
 * A simple read-write lock implementation.
 * - Multiple readers can hold the lock concurrently
 * - Writers have exclusive access
 * - Writers wait for all readers to release
 * - Readers wait if a writer is active or waiting
 */
export class RWLock {
  private readers = 0;
  private writer = false;
  private writerQueue: (() => void)[] = [];
  private readerQueue: (() => void)[] = [];

  async acquireRead(): Promise<void> {
    return new Promise((resolve) => {
      if (!this.writer && this.writerQueue.length === 0) {
        this.readers++;
        resolve();
      } else {
        this.readerQueue.push(() => {
          this.readers++;
          resolve();
        });
      }
    });
  }

  releaseRead(): void {
    this.readers--;
    if (this.readers === 0 && this.writerQueue.length > 0) {
      this.writer = true;
      const next = this.writerQueue.shift();
      if (next) next();
    }
  }

  async acquireWrite(): Promise<void> {
    return new Promise((resolve) => {
      if (!this.writer && this.readers === 0) {
        this.writer = true;
        resolve();
      } else {
        this.writerQueue.push(resolve);
      }
    });
  }

  releaseWrite(): void {
    this.writer = false;
    // Prefer waiting readers over writers to prevent writer starvation
    if (this.readerQueue.length > 0) {
      const readers = this.readerQueue.splice(0);
      for (const reader of readers) {
        reader();
      }
    } else if (this.writerQueue.length > 0) {
      this.writer = true;
      const next = this.writerQueue.shift();
      if (next) next();
    }
  }

  /**
   * Execute a function with read lock
   */
  async withRead<T>(fn: () => T | Promise<T>): Promise<T> {
    await this.acquireRead();
    try {
      return await fn();
    } finally {
      this.releaseRead();
    }
  }

  /**
   * Execute a function with write lock
   */
  async withWrite<T>(fn: () => T | Promise<T>): Promise<T> {
    await this.acquireWrite();
    try {
      return await fn();
    } finally {
      this.releaseWrite();
    }
  }
}

// ============================================================================
// INTERFACE
// ============================================================================

export interface IEdgeRepository {
  create(input: GraphEdgeInput): Promise<GraphEdge>;
  findById(id: string): Promise<GraphEdge | null>;
  findByNodes(fromId: string, toId: string, edgeType?: string): Promise<GraphEdge | null>;
  delete(id: string): Promise<boolean>;
  deleteByNodes(fromId: string, toId: string): Promise<boolean>;
  getEdgesFrom(nodeId: string): Promise<GraphEdge[]>;
  getEdgesTo(nodeId: string): Promise<GraphEdge[]>;
  getAllEdges(nodeId: string): Promise<GraphEdge[]>;
  getRelatedNodeIds(nodeId: string, depth?: number): Promise<string[]>;
  updateWeight(id: string, weight: number): Promise<void>;
  strengthenEdge(id: string, boost: number): Promise<void>;
  pruneWeakEdges(threshold: number): Promise<number>;
  getTransitivePaths(nodeId: string, maxDepth: number): Promise<TransitivePath[]>;
  strengthenConnectedEdges(nodeId: string, boost: number): Promise<number>;
}

// ============================================================================
// ERROR CLASS
// ============================================================================

/**
 * Sanitize error message to prevent sensitive data leakage
 */
function sanitizeErrorMessage(message: string): string {
  let sanitized = message.replace(/\/[^\s]+/g, '[PATH]');
  sanitized = sanitized.replace(/SELECT|INSERT|UPDATE|DELETE|DROP|CREATE/gi, '[SQL]');
  sanitized = sanitized.replace(/\b(password|secret|key|token|auth)\s*[=:]\s*\S+/gi, '[REDACTED]');
  return sanitized;
}

export class EdgeRepositoryError extends Error {
  constructor(
    message: string,
    public readonly code: string,
    cause?: unknown
  ) {
    super(sanitizeErrorMessage(message));
    this.name = 'EdgeRepositoryError';
    if (process.env['NODE_ENV'] === 'development' && cause) {
      this.cause = cause;
    }
  }
}

// ============================================================================
// IMPLEMENTATION
// ============================================================================

export class EdgeRepository implements IEdgeRepository {
  private readonly lock = new RWLock();

  constructor(private readonly db: Database.Database) {}

  /**
   * Create a new edge between two nodes.
   * Handles UNIQUE constraint gracefully by using INSERT OR REPLACE.
   */
  async create(input: GraphEdgeInput): Promise<GraphEdge> {
    return this.lock.withWrite(() => {
      try {
        const id = nanoid();
        const now = new Date().toISOString();
        const weight = input.weight ?? 0.5;

        // Check if edge already exists
        const existing = this.db.prepare(`
          SELECT id FROM graph_edges
          WHERE from_id = ? AND to_id = ? AND edge_type = ?
        `).get(input.fromId, input.toId, input.edgeType) as { id: string } | undefined;

        if (existing) {
          // Update existing edge - boost weight slightly
          const updateStmt = this.db.prepare(`
            UPDATE graph_edges
            SET weight = MIN(1.0, weight + ?),
                metadata = ?
            WHERE id = ?
          `);
          updateStmt.run(weight * 0.1, JSON.stringify(input.metadata || {}), existing.id);

          // Return the updated edge
          const row = this.db.prepare('SELECT * FROM graph_edges WHERE id = ?')
            .get(existing.id) as Record<string, unknown>;
          return this.rowToEdge(row);
        }

        // Insert new edge
        const stmt = this.db.prepare(`
          INSERT INTO graph_edges (
            id, from_id, to_id, edge_type, weight, metadata, created_at
          ) VALUES (?, ?, ?, ?, ?, ?, ?)
        `);

        stmt.run(
          id,
          input.fromId,
          input.toId,
          input.edgeType,
          weight,
          JSON.stringify(input.metadata || {}),
          now
        );

        return {
          id,
          fromId: input.fromId,
          toId: input.toId,
          edgeType: input.edgeType as GraphEdge['edgeType'],
          weight,
          metadata: input.metadata || {},
          createdAt: new Date(now),
        };
      } catch (error) {
        throw new EdgeRepositoryError(
          'Failed to create edge',
          'CREATE_EDGE_FAILED',
          error
        );
      }
    });
  }

  /**
   * Find an edge by its ID.
   */
  async findById(id: string): Promise<GraphEdge | null> {
    return this.lock.withRead(() => {
      try {
        const stmt = this.db.prepare('SELECT * FROM graph_edges WHERE id = ?');
        const row = stmt.get(id) as Record<string, unknown> | undefined;
        if (!row) return null;
        return this.rowToEdge(row);
      } catch (error) {
        throw new EdgeRepositoryError(
          `Failed to find edge: ${id}`,
          'FIND_EDGE_FAILED',
          error
        );
      }
    });
  }

  /**
   * Find an edge by its source and target nodes.
   * Optionally filter by edge type.
   */
  async findByNodes(fromId: string, toId: string, edgeType?: string): Promise<GraphEdge | null> {
    return this.lock.withRead(() => {
      try {
        let stmt;
        let row: Record<string, unknown> | undefined;

        if (edgeType) {
          stmt = this.db.prepare(`
            SELECT * FROM graph_edges
            WHERE from_id = ? AND to_id = ? AND edge_type = ?
          `);
          row = stmt.get(fromId, toId, edgeType) as Record<string, unknown> | undefined;
        } else {
          stmt = this.db.prepare(`
            SELECT * FROM graph_edges
            WHERE from_id = ? AND to_id = ?
          `);
          row = stmt.get(fromId, toId) as Record<string, unknown> | undefined;
        }

        if (!row) return null;
        return this.rowToEdge(row);
      } catch (error) {
        throw new EdgeRepositoryError(
          `Failed to find edge by nodes`,
          'FIND_BY_NODES_FAILED',
          error
        );
      }
    });
  }

  /**
   * Delete an edge by its ID.
   */
  async delete(id: string): Promise<boolean> {
    return this.lock.withWrite(() => {
      try {
        const stmt = this.db.prepare('DELETE FROM graph_edges WHERE id = ?');
        const result = stmt.run(id);
        return result.changes > 0;
      } catch (error) {
        throw new EdgeRepositoryError(
          `Failed to delete edge: ${id}`,
          'DELETE_EDGE_FAILED',
          error
        );
      }
    });
  }

  /**
   * Delete all edges between two nodes (in both directions).
   */
  async deleteByNodes(fromId: string, toId: string): Promise<boolean> {
    return this.lock.withWrite(() => {
      try {
        const stmt = this.db.prepare(`
          DELETE FROM graph_edges
          WHERE (from_id = ? AND to_id = ?) OR (from_id = ? AND to_id = ?)
        `);
        const result = stmt.run(fromId, toId, toId, fromId);
        return result.changes > 0;
      } catch (error) {
        throw new EdgeRepositoryError(
          `Failed to delete edges between nodes`,
          'DELETE_BY_NODES_FAILED',
          error
        );
      }
    });
  }

  /**
   * Get all edges originating from a node.
   */
  async getEdgesFrom(nodeId: string): Promise<GraphEdge[]> {
    return this.lock.withRead(() => {
      try {
        const stmt = this.db.prepare('SELECT * FROM graph_edges WHERE from_id = ?');
        const rows = stmt.all(nodeId) as Record<string, unknown>[];
        return rows.map(row => this.rowToEdge(row));
      } catch (error) {
        throw new EdgeRepositoryError(
          `Failed to get edges from node: ${nodeId}`,
          'GET_EDGES_FROM_FAILED',
          error
        );
      }
    });
  }

  /**
   * Get all edges pointing to a node.
   */
  async getEdgesTo(nodeId: string): Promise<GraphEdge[]> {
    return this.lock.withRead(() => {
      try {
        const stmt = this.db.prepare('SELECT * FROM graph_edges WHERE to_id = ?');
        const rows = stmt.all(nodeId) as Record<string, unknown>[];
        return rows.map(row => this.rowToEdge(row));
      } catch (error) {
        throw new EdgeRepositoryError(
          `Failed to get edges to node: ${nodeId}`,
          'GET_EDGES_TO_FAILED',
          error
        );
      }
    });
  }

  /**
   * Get all edges connected to a node (both incoming and outgoing).
   */
  async getAllEdges(nodeId: string): Promise<GraphEdge[]> {
    return this.lock.withRead(() => {
      try {
        const stmt = this.db.prepare(`
          SELECT * FROM graph_edges
          WHERE from_id = ? OR to_id = ?
        `);
        const rows = stmt.all(nodeId, nodeId) as Record<string, unknown>[];
        return rows.map(row => this.rowToEdge(row));
      } catch (error) {
        throw new EdgeRepositoryError(
          `Failed to get all edges for node: ${nodeId}`,
          'GET_ALL_EDGES_FAILED',
          error
        );
      }
    });
  }

  /**
   * Get related node IDs using BFS traversal.
   * Extracted from database.ts getRelatedNodes().
   */
  async getRelatedNodeIds(nodeId: string, depth: number = 1): Promise<string[]> {
    return this.lock.withRead(() => {
      try {
        const visited = new Set<string>();
        let current = [nodeId];

        for (let d = 0; d < depth; d++) {
          if (current.length === 0) break;

          const placeholders = current.map(() => '?').join(',');
          const stmt = this.db.prepare(`
            SELECT DISTINCT
              CASE WHEN from_id IN (${placeholders}) THEN to_id ELSE from_id END as related_id
            FROM graph_edges
            WHERE from_id IN (${placeholders}) OR to_id IN (${placeholders})
          `);

          const params = [...current, ...current, ...current];
          const rows = stmt.all(...params) as { related_id: string }[];

          const newNodes: string[] = [];
          for (const row of rows) {
            if (!visited.has(row.related_id) && row.related_id !== nodeId) {
              visited.add(row.related_id);
              newNodes.push(row.related_id);
            }
          }
          current = newNodes;
        }

        return Array.from(visited);
      } catch (error) {
        throw new EdgeRepositoryError(
          `Failed to get related nodes: ${nodeId}`,
          'GET_RELATED_FAILED',
          error
        );
      }
    });
  }

  /**
   * Update the weight of an edge.
   */
  async updateWeight(id: string, weight: number): Promise<void> {
    return this.lock.withWrite(() => {
      try {
        // Clamp weight to valid range
        const clampedWeight = Math.max(0, Math.min(1, weight));

        const stmt = this.db.prepare(`
          UPDATE graph_edges SET weight = ? WHERE id = ?
        `);
        stmt.run(clampedWeight, id);
      } catch (error) {
        throw new EdgeRepositoryError(
          `Failed to update edge weight: ${id}`,
          'UPDATE_WEIGHT_FAILED',
          error
        );
      }
    });
  }

  /**
   * Strengthen an edge by boosting its weight.
   * Used for spreading activation.
   */
  async strengthenEdge(id: string, boost: number): Promise<void> {
    return this.lock.withWrite(() => {
      try {
        // Ensure boost is positive and reasonable
        const safeBoost = Math.max(0, Math.min(0.5, boost));

        const stmt = this.db.prepare(`
          UPDATE graph_edges
          SET weight = MIN(1.0, weight + ?)
          WHERE id = ?
        `);
        stmt.run(safeBoost, id);
      } catch (error) {
        throw new EdgeRepositoryError(
          `Failed to strengthen edge: ${id}`,
          'STRENGTHEN_EDGE_FAILED',
          error
        );
      }
    });
  }

  /**
   * Prune edges with weight below a threshold.
   * Returns the number of edges removed.
   */
  async pruneWeakEdges(threshold: number): Promise<number> {
    return this.lock.withWrite(() => {
      try {
        // Validate threshold
        const safeThreshold = Math.max(0, Math.min(1, threshold));

        const stmt = this.db.prepare(`
          DELETE FROM graph_edges WHERE weight < ?
        `);
        const result = stmt.run(safeThreshold);
        return result.changes;
      } catch (error) {
        throw new EdgeRepositoryError(
          'Failed to prune weak edges',
          'PRUNE_EDGES_FAILED',
          error
        );
      }
    });
  }

  /**
   * Get all transitive paths from a node up to maxDepth.
   * Used for spreading activation in graph traversal.
   */
  async getTransitivePaths(nodeId: string, maxDepth: number): Promise<TransitivePath[]> {
    return this.lock.withRead(() => {
      try {
        const paths: TransitivePath[] = [];
        const visited = new Set<string>();

        // BFS with path tracking
        interface QueueItem {
          nodeId: string;
          path: string[];
          totalWeight: number;
        }

        const queue: QueueItem[] = [{ nodeId, path: [nodeId], totalWeight: 1.0 }];
        visited.add(nodeId);

        while (queue.length > 0) {
          const current = queue.shift()!;

          if (current.path.length > maxDepth + 1) continue;

          // Get all connected edges
          const stmt = this.db.prepare(`
            SELECT to_id, from_id, weight FROM graph_edges
            WHERE from_id = ? OR to_id = ?
          `);
          const edges = stmt.all(current.nodeId, current.nodeId) as {
            to_id: string;
            from_id: string;
            weight: number;
          }[];

          for (const edge of edges) {
            const nextNode = edge.from_id === current.nodeId ? edge.to_id : edge.from_id;

            if (!visited.has(nextNode)) {
              visited.add(nextNode);
              const newPath = [...current.path, nextNode];
              const newWeight = current.totalWeight * edge.weight;

              paths.push({ path: newPath, totalWeight: newWeight });

              if (newPath.length <= maxDepth) {
                queue.push({
                  nodeId: nextNode,
                  path: newPath,
                  totalWeight: newWeight,
                });
              }
            }
          }
        }

        // Sort by total weight (descending) for relevance
        return paths.sort((a, b) => b.totalWeight - a.totalWeight);
      } catch (error) {
        throw new EdgeRepositoryError(
          `Failed to get transitive paths: ${nodeId}`,
          'GET_PATHS_FAILED',
          error
        );
      }
    });
  }

  /**
   * Strengthen all edges connected to a node.
   * Used for memory reconsolidation.
   * Returns the number of edges strengthened.
   */
  async strengthenConnectedEdges(nodeId: string, boost: number): Promise<number> {
    return this.lock.withWrite(() => {
      try {
        // Ensure boost is positive and reasonable
        const safeBoost = Math.max(0, Math.min(0.5, boost));

        const stmt = this.db.prepare(`
          UPDATE graph_edges
          SET weight = MIN(1.0, weight + ?)
          WHERE from_id = ? OR to_id = ?
        `);
        const result = stmt.run(safeBoost, nodeId, nodeId);
        return result.changes;
      } catch (error) {
        throw new EdgeRepositoryError(
          `Failed to strengthen connected edges: ${nodeId}`,
          'STRENGTHEN_CONNECTED_FAILED',
          error
        );
      }
    });
  }

  // ============================================================================
  // HELPERS
  // ============================================================================

  private rowToEdge(row: Record<string, unknown>): GraphEdge {
    return {
      id: row['id'] as string,
      fromId: row['from_id'] as string,
      toId: row['to_id'] as string,
      edgeType: row['edge_type'] as GraphEdge['edgeType'],
      weight: row['weight'] as number,
      metadata: this.safeJsonParse(row['metadata'] as string, {}),
      createdAt: new Date(row['created_at'] as string),
    };
  }

  private safeJsonParse<T>(value: string | null | undefined, fallback: T): T {
    if (!value) return fallback;
    try {
      return JSON.parse(value) as T;
    } catch {
      return fallback;
    }
  }
}
