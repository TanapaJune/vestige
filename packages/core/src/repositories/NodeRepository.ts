/**
 * NodeRepository - Repository for knowledge node operations
 *
 * Extracted from the monolithic database.ts to provide a focused, testable
 * interface for node CRUD operations with proper concurrency control.
 */

import type Database from 'better-sqlite3';
import { nanoid } from 'nanoid';
import type {
  KnowledgeNode,
  KnowledgeNodeInput,
} from '../core/types.js';
import { RWLock } from '../utils/mutex.js';
import { safeJsonParse } from '../utils/json.js';
import { NotFoundError, ValidationError, DatabaseError } from '../core/errors.js';
import { analyzeSentimentIntensity, captureGitContext } from '../core/database.js';

// ============================================================================
// CONSTANTS
// ============================================================================

const DEFAULT_LIMIT = 50;
const MAX_LIMIT = 500;

// Input validation limits
const MAX_CONTENT_LENGTH = 1_000_000; // 1MB max content
const MAX_QUERY_LENGTH = 10_000; // 10KB max query
const MAX_TAGS_COUNT = 100; // Max tags per node

// SM-2 Spaced Repetition Constants
const SM2_EASE_FACTOR = 2.5;
const SM2_LAPSE_THRESHOLD = 0.3;
const SM2_MIN_STABILITY = 1.0;
const SM2_MAX_STABILITY = 365.0;

// Sentiment-Weighted Decay Constants
const SENTIMENT_STABILITY_BOOST = 2.0;
const SENTIMENT_MIN_BOOST = 1.0;

// ============================================================================
// TYPES
// ============================================================================

export interface PaginationOptions {
  limit?: number;
  offset?: number;
}

export interface PaginatedResult<T> {
  items: T[];
  total: number;
  limit: number;
  offset: number;
  hasMore: boolean;
}

export interface GitContext {
  branch?: string;
  commit?: string;
  commitMessage?: string;
  repoPath?: string;
  dirty?: boolean;
  changedFiles?: string[];
}

// ============================================================================
// INTERFACE
// ============================================================================

export interface INodeRepository {
  findById(id: string): Promise<KnowledgeNode | null>;
  findByIds(ids: string[]): Promise<KnowledgeNode[]>;
  create(input: KnowledgeNodeInput): Promise<KnowledgeNode>;
  update(id: string, updates: Partial<KnowledgeNodeInput>): Promise<KnowledgeNode | null>;
  delete(id: string): Promise<boolean>;
  search(query: string, options?: PaginationOptions): Promise<PaginatedResult<KnowledgeNode>>;
  getRecent(options?: PaginationOptions): Promise<PaginatedResult<KnowledgeNode>>;
  getDecaying(threshold: number, options?: PaginationOptions): Promise<PaginatedResult<KnowledgeNode>>;
  getDueForReview(options?: PaginationOptions): Promise<PaginatedResult<KnowledgeNode>>;
  recordAccess(id: string): Promise<void>;
  markReviewed(id: string): Promise<KnowledgeNode>;
  applyDecay(id: string): Promise<number>;
  applyDecayAll(): Promise<number>;
  findByTag(tag: string, options?: PaginationOptions): Promise<PaginatedResult<KnowledgeNode>>;
  findByPerson(personName: string, options?: PaginationOptions): Promise<PaginatedResult<KnowledgeNode>>;
}

// ============================================================================
// VALIDATION HELPERS
// ============================================================================

/**
 * Validate string length for inputs
 */
function validateStringLength(value: string, maxLength: number, fieldName: string): void {
  if (value && value.length > maxLength) {
    throw new ValidationError(
      `${fieldName} exceeds maximum length of ${maxLength} characters`,
      { field: fieldName.toLowerCase(), maxLength, actualLength: value.length }
    );
  }
}

/**
 * Validate array length for inputs
 */
function validateArrayLength<T>(arr: T[] | undefined, maxLength: number, fieldName: string): void {
  if (arr && arr.length > maxLength) {
    throw new ValidationError(
      `${fieldName} exceeds maximum count of ${maxLength} items`,
      { field: fieldName.toLowerCase(), maxLength, actualLength: arr.length }
    );
  }
}

/**
 * Normalize pagination options
 */
function normalizePagination(options: PaginationOptions = {}): { limit: number; offset: number } {
  const { limit = DEFAULT_LIMIT, offset = 0 } = options;
  return {
    limit: Math.min(Math.max(1, limit), MAX_LIMIT),
    offset: Math.max(0, offset),
  };
}

// ============================================================================
// IMPLEMENTATION
// ============================================================================

export class NodeRepository implements INodeRepository {
  private readonly lock = new RWLock();

  constructor(private readonly db: Database.Database) {}

  // --------------------------------------------------------------------------
  // READ OPERATIONS
  // --------------------------------------------------------------------------

  async findById(id: string): Promise<KnowledgeNode | null> {
    return this.lock.withReadLock(async () => {
      try {
        const stmt = this.db.prepare('SELECT * FROM knowledge_nodes WHERE id = ?');
        const row = stmt.get(id) as Record<string, unknown> | undefined;
        if (!row) return null;
        return this.rowToEntity(row);
      } catch (error) {
        throw new DatabaseError(`Failed to get node: ${id}`, error);
      }
    });
  }

  async findByIds(ids: string[]): Promise<KnowledgeNode[]> {
    if (ids.length === 0) return [];

    return this.lock.withReadLock(async () => {
      try {
        const placeholders = ids.map(() => '?').join(',');
        const stmt = this.db.prepare(
          `SELECT * FROM knowledge_nodes WHERE id IN (${placeholders})`
        );
        const rows = stmt.all(...ids) as Record<string, unknown>[];
        return rows.map((row) => this.rowToEntity(row));
      } catch (error) {
        throw new DatabaseError('Failed to get nodes by IDs', error);
      }
    });
  }

  async search(query: string, options: PaginationOptions = {}): Promise<PaginatedResult<KnowledgeNode>> {
    return this.lock.withReadLock(async () => {
      try {
        // Input validation
        validateStringLength(query, MAX_QUERY_LENGTH, 'Search query');

        // Sanitize FTS5 query to prevent injection
        const sanitizedQuery = query
          .replace(/[^\w\s\-]/g, ' ')
          .trim();

        if (!sanitizedQuery) {
          return {
            items: [],
            total: 0,
            limit: DEFAULT_LIMIT,
            offset: 0,
            hasMore: false,
          };
        }

        const { limit, offset } = normalizePagination(options);

        // Get total count
        const countStmt = this.db.prepare(`
          SELECT COUNT(*) as total FROM knowledge_nodes kn
          JOIN knowledge_fts fts ON kn.id = fts.id
          WHERE knowledge_fts MATCH ?
        `);
        const countResult = countStmt.get(sanitizedQuery) as { total: number };
        const total = countResult.total;

        // Get paginated results
        const stmt = this.db.prepare(`
          SELECT kn.* FROM knowledge_nodes kn
          JOIN knowledge_fts fts ON kn.id = fts.id
          WHERE knowledge_fts MATCH ?
          ORDER BY rank
          LIMIT ? OFFSET ?
        `);
        const rows = stmt.all(sanitizedQuery, limit, offset) as Record<string, unknown>[];
        const items = rows.map((row) => this.rowToEntity(row));

        return {
          items,
          total,
          limit,
          offset,
          hasMore: offset + items.length < total,
        };
      } catch (error) {
        if (error instanceof ValidationError) throw error;
        throw new DatabaseError('Search operation failed', error);
      }
    });
  }

  async getRecent(options: PaginationOptions = {}): Promise<PaginatedResult<KnowledgeNode>> {
    return this.lock.withReadLock(async () => {
      try {
        const { limit, offset } = normalizePagination(options);

        // Get total count
        const countResult = this.db.prepare('SELECT COUNT(*) as total FROM knowledge_nodes').get() as {
          total: number;
        };
        const total = countResult.total;

        // Get paginated results
        const stmt = this.db.prepare(`
          SELECT * FROM knowledge_nodes
          ORDER BY created_at DESC
          LIMIT ? OFFSET ?
        `);
        const rows = stmt.all(limit, offset) as Record<string, unknown>[];
        const items = rows.map((row) => this.rowToEntity(row));

        return {
          items,
          total,
          limit,
          offset,
          hasMore: offset + items.length < total,
        };
      } catch (error) {
        throw new DatabaseError('Failed to get recent nodes', error);
      }
    });
  }

  async getDecaying(
    threshold: number = 0.5,
    options: PaginationOptions = {}
  ): Promise<PaginatedResult<KnowledgeNode>> {
    return this.lock.withReadLock(async () => {
      try {
        const { limit, offset } = normalizePagination(options);

        // Get total count
        const countStmt = this.db.prepare(`
          SELECT COUNT(*) as total FROM knowledge_nodes
          WHERE retention_strength < ?
        `);
        const countResult = countStmt.get(threshold) as { total: number };
        const total = countResult.total;

        // Get paginated results
        const stmt = this.db.prepare(`
          SELECT * FROM knowledge_nodes
          WHERE retention_strength < ?
          ORDER BY retention_strength ASC
          LIMIT ? OFFSET ?
        `);
        const rows = stmt.all(threshold, limit, offset) as Record<string, unknown>[];
        const items = rows.map((row) => this.rowToEntity(row));

        return {
          items,
          total,
          limit,
          offset,
          hasMore: offset + items.length < total,
        };
      } catch (error) {
        throw new DatabaseError('Failed to get decaying nodes', error);
      }
    });
  }

  async getDueForReview(options: PaginationOptions = {}): Promise<PaginatedResult<KnowledgeNode>> {
    return this.lock.withReadLock(async () => {
      try {
        const { limit, offset } = normalizePagination(options);
        const now = new Date().toISOString();

        // Get total count
        const countStmt = this.db.prepare(`
          SELECT COUNT(*) as total FROM knowledge_nodes
          WHERE next_review_date IS NOT NULL AND next_review_date <= ?
        `);
        const countResult = countStmt.get(now) as { total: number };
        const total = countResult.total;

        // Get paginated results, ordered by retention strength (most urgent first)
        const stmt = this.db.prepare(`
          SELECT * FROM knowledge_nodes
          WHERE next_review_date IS NOT NULL AND next_review_date <= ?
          ORDER BY retention_strength ASC, next_review_date ASC
          LIMIT ? OFFSET ?
        `);
        const rows = stmt.all(now, limit, offset) as Record<string, unknown>[];
        const items = rows.map((row) => this.rowToEntity(row));

        return {
          items,
          total,
          limit,
          offset,
          hasMore: offset + items.length < total,
        };
      } catch (error) {
        throw new DatabaseError('Failed to get nodes due for review', error);
      }
    });
  }

  async findByTag(tag: string, options: PaginationOptions = {}): Promise<PaginatedResult<KnowledgeNode>> {
    return this.lock.withReadLock(async () => {
      try {
        const { limit, offset } = normalizePagination(options);

        // Escape special JSON/LIKE characters
        const escapedTag = tag
          .replace(/\\/g, '\\\\')
          .replace(/%/g, '\\%')
          .replace(/_/g, '\\_')
          .replace(/"/g, '\\"');

        // Get total count
        const countStmt = this.db.prepare(`
          SELECT COUNT(*) as total FROM knowledge_nodes
          WHERE tags LIKE ? ESCAPE '\\'
        `);
        const countResult = countStmt.get(`%"${escapedTag}"%`) as { total: number };
        const total = countResult.total;

        // Get paginated results
        const stmt = this.db.prepare(`
          SELECT * FROM knowledge_nodes
          WHERE tags LIKE ? ESCAPE '\\'
          ORDER BY created_at DESC
          LIMIT ? OFFSET ?
        `);
        const rows = stmt.all(`%"${escapedTag}"%`, limit, offset) as Record<string, unknown>[];
        const items = rows.map((row) => this.rowToEntity(row));

        return {
          items,
          total,
          limit,
          offset,
          hasMore: offset + items.length < total,
        };
      } catch (error) {
        throw new DatabaseError('Failed to find nodes by tag', error);
      }
    });
  }

  async findByPerson(
    personName: string,
    options: PaginationOptions = {}
  ): Promise<PaginatedResult<KnowledgeNode>> {
    return this.lock.withReadLock(async () => {
      try {
        const { limit, offset } = normalizePagination(options);

        // Escape special JSON/LIKE characters
        const escapedPerson = personName
          .replace(/\\/g, '\\\\')
          .replace(/%/g, '\\%')
          .replace(/_/g, '\\_')
          .replace(/"/g, '\\"');

        // Get total count
        const countStmt = this.db.prepare(`
          SELECT COUNT(*) as total FROM knowledge_nodes
          WHERE people LIKE ? ESCAPE '\\'
        `);
        const countResult = countStmt.get(`%"${escapedPerson}"%`) as { total: number };
        const total = countResult.total;

        // Get paginated results
        const stmt = this.db.prepare(`
          SELECT * FROM knowledge_nodes
          WHERE people LIKE ? ESCAPE '\\'
          ORDER BY created_at DESC
          LIMIT ? OFFSET ?
        `);
        const rows = stmt.all(`%"${escapedPerson}"%`, limit, offset) as Record<string, unknown>[];
        const items = rows.map((row) => this.rowToEntity(row));

        return {
          items,
          total,
          limit,
          offset,
          hasMore: offset + items.length < total,
        };
      } catch (error) {
        throw new DatabaseError('Failed to find nodes by person', error);
      }
    });
  }

  // --------------------------------------------------------------------------
  // WRITE OPERATIONS
  // --------------------------------------------------------------------------

  async create(input: KnowledgeNodeInput): Promise<KnowledgeNode> {
    return this.lock.withWriteLock(async () => {
      try {
        // Input validation
        validateStringLength(input.content, MAX_CONTENT_LENGTH, 'Content');
        validateStringLength(input.summary || '', MAX_CONTENT_LENGTH, 'Summary');
        validateArrayLength(input.tags, MAX_TAGS_COUNT, 'Tags');
        validateArrayLength(input.people, MAX_TAGS_COUNT, 'People');
        validateArrayLength(input.concepts, MAX_TAGS_COUNT, 'Concepts');
        validateArrayLength(input.events, MAX_TAGS_COUNT, 'Events');

        // Validate confidence is within bounds
        const confidence = Math.max(0, Math.min(1, input.confidence ?? 0.8));
        const retention = Math.max(0, Math.min(1, input.retentionStrength ?? 1.0));

        // Analyze emotional intensity of content
        const sentimentIntensity =
          input.sentimentIntensity ?? analyzeSentimentIntensity(input.content);

        // Git-Blame for Thoughts: Capture current code context
        const gitContext = input.gitContext ?? captureGitContext();

        const id = nanoid();
        const now = new Date().toISOString();

        const stmt = this.db.prepare(`
          INSERT INTO knowledge_nodes (
            id, content, summary,
            created_at, updated_at, last_accessed_at, access_count,
            retention_strength, sentiment_intensity, next_review_date, review_count,
            source_type, source_platform, source_id, source_url, source_chain, git_context,
            confidence, is_contradicted, contradiction_ids,
            people, concepts, events, tags
          ) VALUES (
            ?, ?, ?,
            ?, ?, ?, ?,
            ?, ?, ?, ?,
            ?, ?, ?, ?, ?, ?,
            ?, ?, ?,
            ?, ?, ?, ?
          )
        `);

        const createdAt = input.createdAt instanceof Date
          ? input.createdAt.toISOString()
          : (input.createdAt || now);

        stmt.run(
          id,
          input.content,
          input.summary || null,
          createdAt,
          now,
          now,
          0,
          retention,
          sentimentIntensity,
          input.nextReviewDate instanceof Date
            ? input.nextReviewDate.toISOString()
            : (input.nextReviewDate || null),
          0,
          input.sourceType,
          input.sourcePlatform,
          input.sourceId || null,
          input.sourceUrl || null,
          JSON.stringify(input.sourceChain || []),
          gitContext ? JSON.stringify(gitContext) : null,
          confidence,
          input.isContradicted ? 1 : 0,
          JSON.stringify(input.contradictionIds || []),
          JSON.stringify(input.people || []),
          JSON.stringify(input.concepts || []),
          JSON.stringify(input.events || []),
          JSON.stringify(input.tags || [])
        );

        // Return the created node
        const node = await this.findById(id);
        if (!node) {
          throw new DatabaseError('Failed to retrieve created node');
        }
        return node;
      } catch (error) {
        if (error instanceof ValidationError || error instanceof DatabaseError) throw error;
        throw new DatabaseError('Failed to insert knowledge node', error);
      }
    });
  }

  async update(id: string, updates: Partial<KnowledgeNodeInput>): Promise<KnowledgeNode | null> {
    return this.lock.withWriteLock(async () => {
      try {
        // Check if node exists
        const existing = this.db.prepare('SELECT * FROM knowledge_nodes WHERE id = ?').get(id);
        if (!existing) {
          return null;
        }

        // Input validation
        if (updates.content !== undefined) {
          validateStringLength(updates.content, MAX_CONTENT_LENGTH, 'Content');
        }
        if (updates.summary !== undefined) {
          validateStringLength(updates.summary, MAX_CONTENT_LENGTH, 'Summary');
        }
        if (updates.tags !== undefined) {
          validateArrayLength(updates.tags, MAX_TAGS_COUNT, 'Tags');
        }
        if (updates.people !== undefined) {
          validateArrayLength(updates.people, MAX_TAGS_COUNT, 'People');
        }
        if (updates.concepts !== undefined) {
          validateArrayLength(updates.concepts, MAX_TAGS_COUNT, 'Concepts');
        }
        if (updates.events !== undefined) {
          validateArrayLength(updates.events, MAX_TAGS_COUNT, 'Events');
        }

        // Build dynamic update
        const setClauses: string[] = [];
        const values: unknown[] = [];

        if (updates.content !== undefined) {
          setClauses.push('content = ?');
          values.push(updates.content);

          // Re-analyze sentiment when content changes
          const sentimentIntensity = analyzeSentimentIntensity(updates.content);
          setClauses.push('sentiment_intensity = ?');
          values.push(sentimentIntensity);
        }

        if (updates.summary !== undefined) {
          setClauses.push('summary = ?');
          values.push(updates.summary);
        }

        if (updates.confidence !== undefined) {
          setClauses.push('confidence = ?');
          values.push(Math.max(0, Math.min(1, updates.confidence)));
        }

        if (updates.retentionStrength !== undefined) {
          setClauses.push('retention_strength = ?');
          values.push(Math.max(0, Math.min(1, updates.retentionStrength)));
        }

        if (updates.tags !== undefined) {
          setClauses.push('tags = ?');
          values.push(JSON.stringify(updates.tags));
        }

        if (updates.people !== undefined) {
          setClauses.push('people = ?');
          values.push(JSON.stringify(updates.people));
        }

        if (updates.concepts !== undefined) {
          setClauses.push('concepts = ?');
          values.push(JSON.stringify(updates.concepts));
        }

        if (updates.events !== undefined) {
          setClauses.push('events = ?');
          values.push(JSON.stringify(updates.events));
        }

        if (updates.isContradicted !== undefined) {
          setClauses.push('is_contradicted = ?');
          values.push(updates.isContradicted ? 1 : 0);
        }

        if (updates.contradictionIds !== undefined) {
          setClauses.push('contradiction_ids = ?');
          values.push(JSON.stringify(updates.contradictionIds));
        }

        if (setClauses.length === 0) {
          // No updates to make, just return existing node
          return this.rowToEntity(existing as Record<string, unknown>);
        }

        // Always update updated_at
        setClauses.push('updated_at = ?');
        values.push(new Date().toISOString());

        // Add the ID for the WHERE clause
        values.push(id);

        const sql = `UPDATE knowledge_nodes SET ${setClauses.join(', ')} WHERE id = ?`;
        this.db.prepare(sql).run(...values);

        // Return updated node
        const updated = this.db.prepare('SELECT * FROM knowledge_nodes WHERE id = ?').get(id);
        return updated ? this.rowToEntity(updated as Record<string, unknown>) : null;
      } catch (error) {
        if (error instanceof ValidationError || error instanceof DatabaseError) throw error;
        throw new DatabaseError(`Failed to update node: ${id}`, error);
      }
    });
  }

  async delete(id: string): Promise<boolean> {
    return this.lock.withWriteLock(async () => {
      try {
        const stmt = this.db.prepare('DELETE FROM knowledge_nodes WHERE id = ?');
        const result = stmt.run(id);
        return result.changes > 0;
      } catch (error) {
        throw new DatabaseError(`Failed to delete node: ${id}`, error);
      }
    });
  }

  async recordAccess(id: string): Promise<void> {
    return this.lock.withWriteLock(async () => {
      try {
        const stmt = this.db.prepare(`
          UPDATE knowledge_nodes
          SET last_accessed_at = ?, access_count = access_count + 1
          WHERE id = ?
        `);
        stmt.run(new Date().toISOString(), id);
      } catch (error) {
        throw new DatabaseError(`Failed to record access: ${id}`, error);
      }
    });
  }

  async markReviewed(id: string): Promise<KnowledgeNode> {
    return this.lock.withWriteLock(async () => {
      try {
        // Get the node first
        const nodeStmt = this.db.prepare('SELECT * FROM knowledge_nodes WHERE id = ?');
        const nodeRow = nodeStmt.get(id) as Record<string, unknown> | undefined;

        if (!nodeRow) {
          throw new NotFoundError('KnowledgeNode', id);
        }

        const node = this.rowToEntity(nodeRow);
        const currentStability = node.stabilityFactor ?? SM2_MIN_STABILITY;
        let newStability: number;
        let newReviewCount: number;

        // SM-2 with Lapse Detection
        if (node.retentionStrength >= SM2_LAPSE_THRESHOLD) {
          // SUCCESSFUL RECALL: Memory was still accessible
          newStability = Math.min(SM2_MAX_STABILITY, currentStability * SM2_EASE_FACTOR);
          newReviewCount = node.reviewCount + 1;
        } else {
          // LAPSE: Memory had decayed too far
          newStability = SM2_MIN_STABILITY;
          newReviewCount = node.reviewCount + 1;
        }

        // Reset retention to full strength
        const newRetention = 1.0;

        // Calculate next review date
        const daysUntilReview = Math.ceil(newStability);
        const nextReview = new Date();
        nextReview.setDate(nextReview.getDate() + daysUntilReview);

        const updateStmt = this.db.prepare(`
          UPDATE knowledge_nodes
          SET retention_strength = ?,
              stability_factor = ?,
              review_count = ?,
              next_review_date = ?,
              last_accessed_at = ?,
              updated_at = ?
          WHERE id = ?
        `);
        const now = new Date().toISOString();
        updateStmt.run(
          newRetention,
          newStability,
          newReviewCount,
          nextReview.toISOString(),
          now,
          now,
          id
        );

        // Return the updated node
        const updatedRow = nodeStmt.get(id) as Record<string, unknown>;
        return this.rowToEntity(updatedRow);
      } catch (error) {
        if (error instanceof NotFoundError) throw error;
        throw new DatabaseError('Failed to mark node as reviewed', error);
      }
    });
  }

  async applyDecay(id: string): Promise<number> {
    return this.lock.withWriteLock(async () => {
      try {
        const nodeStmt = this.db.prepare(`
          SELECT id, last_accessed_at, retention_strength, stability_factor, sentiment_intensity
          FROM knowledge_nodes WHERE id = ?
        `);
        const node = nodeStmt.get(id) as {
          id: string;
          last_accessed_at: string;
          retention_strength: number;
          stability_factor: number | null;
          sentiment_intensity: number | null;
        } | undefined;

        if (!node) {
          throw new NotFoundError('KnowledgeNode', id);
        }

        const now = Date.now();
        const lastAccessed = new Date(node.last_accessed_at).getTime();
        const daysSince = (now - lastAccessed) / (1000 * 60 * 60 * 24);

        const baseStability = node.stability_factor ?? SM2_MIN_STABILITY;
        const sentimentIntensity = node.sentiment_intensity ?? 0;
        const sentimentMultiplier =
          SENTIMENT_MIN_BOOST + sentimentIntensity * (SENTIMENT_STABILITY_BOOST - SENTIMENT_MIN_BOOST);
        const effectiveStability = baseStability * sentimentMultiplier;

        // Ebbinghaus forgetting curve: R = e^(-t/S)
        const newRetention = Math.max(0.1, node.retention_strength * Math.exp(-daysSince / effectiveStability));

        const updateStmt = this.db.prepare(`
          UPDATE knowledge_nodes SET retention_strength = ? WHERE id = ?
        `);
        updateStmt.run(newRetention, id);

        return newRetention;
      } catch (error) {
        if (error instanceof NotFoundError) throw error;
        throw new DatabaseError(`Failed to apply decay to node: ${id}`, error);
      }
    });
  }

  async applyDecayAll(): Promise<number> {
    return this.lock.withWriteLock(async () => {
      try {
        const now = Date.now();

        // Use IMMEDIATE transaction for consistency
        const transaction = this.db.transaction(() => {
          const nodes = this.db
            .prepare(
              `
            SELECT id, last_accessed_at, retention_strength, stability_factor, sentiment_intensity
            FROM knowledge_nodes
          `
            )
            .all() as {
            id: string;
            last_accessed_at: string;
            retention_strength: number;
            stability_factor: number | null;
            sentiment_intensity: number | null;
          }[];

          let updated = 0;
          const updateStmt = this.db.prepare(`
            UPDATE knowledge_nodes SET retention_strength = ? WHERE id = ?
          `);

          for (const node of nodes) {
            const lastAccessed = new Date(node.last_accessed_at).getTime();
            const daysSince = (now - lastAccessed) / (1000 * 60 * 60 * 24);

            const baseStability = node.stability_factor ?? SM2_MIN_STABILITY;
            const sentimentIntensity = node.sentiment_intensity ?? 0;
            const sentimentMultiplier =
              SENTIMENT_MIN_BOOST +
              sentimentIntensity * (SENTIMENT_STABILITY_BOOST - SENTIMENT_MIN_BOOST);
            const effectiveStability = baseStability * sentimentMultiplier;

            const newRetention = Math.max(
              0.1,
              node.retention_strength * Math.exp(-daysSince / effectiveStability)
            );

            if (Math.abs(newRetention - node.retention_strength) > 0.01) {
              updateStmt.run(newRetention, node.id);
              updated++;
            }
          }

          return updated;
        });

        return transaction.immediate();
      } catch (error) {
        throw new DatabaseError('Failed to apply decay to all nodes', error);
      }
    });
  }

  // --------------------------------------------------------------------------
  // PRIVATE HELPERS
  // --------------------------------------------------------------------------

  /**
   * Convert a database row to a KnowledgeNode entity
   */
  private rowToEntity(row: Record<string, unknown>): KnowledgeNode {
    // Parse git context separately with proper null handling
    let gitContext: GitContext | undefined;
    if (row['git_context']) {
      const parsed = safeJsonParse<GitContext | null>(row['git_context'] as string, null);
      if (parsed !== null) {
        gitContext = parsed;
      }
    }

    return {
      id: row['id'] as string,
      content: row['content'] as string,
      summary: row['summary'] as string | undefined,
      createdAt: new Date(row['created_at'] as string),
      updatedAt: new Date(row['updated_at'] as string),
      lastAccessedAt: new Date(row['last_accessed_at'] as string),
      accessCount: row['access_count'] as number,
      retentionStrength: row['retention_strength'] as number,
      stabilityFactor: (row['stability_factor'] as number) ?? SM2_MIN_STABILITY,
      sentimentIntensity: (row['sentiment_intensity'] as number) ?? 0,
      // Dual-strength memory model fields
      storageStrength: (row['storage_strength'] as number) ?? 1,
      retrievalStrength: (row['retrieval_strength'] as number) ?? 1,
      nextReviewDate: row['next_review_date']
        ? new Date(row['next_review_date'] as string)
        : undefined,
      reviewCount: row['review_count'] as number,
      sourceType: row['source_type'] as KnowledgeNode['sourceType'],
      sourcePlatform: row['source_platform'] as KnowledgeNode['sourcePlatform'],
      sourceId: row['source_id'] as string | undefined,
      sourceUrl: row['source_url'] as string | undefined,
      sourceChain: safeJsonParse<string[]>(row['source_chain'] as string, []),
      gitContext,
      confidence: row['confidence'] as number,
      isContradicted: Boolean(row['is_contradicted']),
      contradictionIds: safeJsonParse<string[]>(row['contradiction_ids'] as string, []),
      people: safeJsonParse<string[]>(row['people'] as string, []),
      concepts: safeJsonParse<string[]>(row['concepts'] as string, []),
      events: safeJsonParse<string[]>(row['events'] as string, []),
      tags: safeJsonParse<string[]>(row['tags'] as string, []),
    };
  }
}
