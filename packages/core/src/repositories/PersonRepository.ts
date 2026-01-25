import Database from 'better-sqlite3';
import { nanoid } from 'nanoid';
import type { PersonNode } from '../core/types.js';

// ============================================================================
// CONSTANTS
// ============================================================================

const DEFAULT_LIMIT = 50;
const MAX_LIMIT = 500;
const MAX_NAME_LENGTH = 500;
const MAX_CONTENT_LENGTH = 1_000_000;
const MAX_ARRAY_COUNT = 100;

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

export interface PersonNodeInput {
  name: string;
  aliases?: string[];
  howWeMet?: string;
  relationshipType?: string;
  organization?: string;
  role?: string;
  location?: string;
  email?: string;
  phone?: string;
  socialLinks?: Record<string, string>;
  preferredChannel?: string;
  sharedTopics?: string[];
  sharedProjects?: string[];
  notes?: string;
  relationshipHealth?: number;
  lastContactAt?: Date;
  contactFrequency?: number;
}

// ============================================================================
// ERROR TYPE
// ============================================================================

export class PersonRepositoryError extends Error {
  constructor(
    message: string,
    public readonly code: string,
    cause?: unknown
  ) {
    super(sanitizeErrorMessage(message));
    this.name = 'PersonRepositoryError';
    if (process.env['NODE_ENV'] === 'development' && cause) {
      this.cause = cause;
    }
  }
}

// ============================================================================
// HELPER FUNCTIONS
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

/**
 * Safe JSON parse with fallback - never throws
 */
function safeJsonParse<T>(value: string | null | undefined, fallback: T): T {
  if (!value) return fallback;
  try {
    const parsed = JSON.parse(value);
    if (typeof parsed !== typeof fallback) {
      return fallback;
    }
    return parsed as T;
  } catch {
    return fallback;
  }
}

/**
 * Validate string length for inputs
 */
function validateStringLength(value: string | undefined, maxLength: number, fieldName: string): void {
  if (value && value.length > maxLength) {
    throw new PersonRepositoryError(
      `${fieldName} exceeds maximum length of ${maxLength} characters`,
      'INPUT_TOO_LONG'
    );
  }
}

/**
 * Validate array length for inputs
 */
function validateArrayLength<T>(arr: T[] | undefined, maxLength: number, fieldName: string): void {
  if (arr && arr.length > maxLength) {
    throw new PersonRepositoryError(
      `${fieldName} exceeds maximum count of ${maxLength} items`,
      'INPUT_TOO_MANY_ITEMS'
    );
  }
}

// ============================================================================
// READ-WRITE LOCK
// ============================================================================

/**
 * A simple read-write lock for concurrent access control.
 * Allows multiple readers or a single writer, but not both.
 */
export class RWLock {
  private readers = 0;
  private writer = false;
  private readQueue: (() => void)[] = [];
  private writeQueue: (() => void)[] = [];

  /**
   * Acquire a read lock. Multiple readers can hold the lock simultaneously.
   */
  async acquireRead(): Promise<void> {
    return new Promise((resolve) => {
      if (!this.writer && this.writeQueue.length === 0) {
        this.readers++;
        resolve();
      } else {
        this.readQueue.push(() => {
          this.readers++;
          resolve();
        });
      }
    });
  }

  /**
   * Release a read lock.
   */
  releaseRead(): void {
    this.readers--;
    if (this.readers === 0) {
      this.processWriteQueue();
    }
  }

  /**
   * Acquire a write lock. Only one writer can hold the lock at a time.
   */
  async acquireWrite(): Promise<void> {
    return new Promise((resolve) => {
      if (!this.writer && this.readers === 0) {
        this.writer = true;
        resolve();
      } else {
        this.writeQueue.push(() => {
          this.writer = true;
          resolve();
        });
      }
    });
  }

  /**
   * Release a write lock.
   */
  releaseWrite(): void {
    this.writer = false;
    // Process read queue first to prevent writer starvation
    this.processReadQueue();
    if (this.readers === 0) {
      this.processWriteQueue();
    }
  }

  private processReadQueue(): void {
    while (this.readQueue.length > 0 && !this.writer) {
      const next = this.readQueue.shift();
      if (next) next();
    }
  }

  private processWriteQueue(): void {
    if (this.writeQueue.length > 0 && this.readers === 0 && !this.writer) {
      const next = this.writeQueue.shift();
      if (next) next();
    }
  }

  /**
   * Execute a function with a read lock.
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
   * Execute a function with a write lock.
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

export interface IPersonRepository {
  findById(id: string): Promise<PersonNode | null>;
  findByName(name: string): Promise<PersonNode | null>;
  searchByName(query: string, options?: PaginationOptions): Promise<PaginatedResult<PersonNode>>;
  create(input: PersonNodeInput): Promise<PersonNode>;
  update(id: string, updates: Partial<PersonNodeInput>): Promise<PersonNode | null>;
  delete(id: string): Promise<boolean>;
  getPeopleToReconnect(daysSinceContact: number, options?: PaginationOptions): Promise<PaginatedResult<PersonNode>>;
  recordContact(id: string): Promise<void>;
  findByOrganization(org: string, options?: PaginationOptions): Promise<PaginatedResult<PersonNode>>;
  findBySharedTopic(topic: string, options?: PaginationOptions): Promise<PaginatedResult<PersonNode>>;
  getAll(options?: PaginationOptions): Promise<PaginatedResult<PersonNode>>;
}

// ============================================================================
// IMPLEMENTATION
// ============================================================================

export class PersonRepository implements IPersonRepository {
  private readonly lock = new RWLock();

  constructor(private readonly db: Database.Database) {}

  /**
   * Convert a database row to a PersonNode entity.
   */
  private rowToEntity(row: Record<string, unknown>): PersonNode {
    return {
      id: row['id'] as string,
      name: row['name'] as string,
      aliases: safeJsonParse<string[]>(row['aliases'] as string, []),
      howWeMet: row['how_we_met'] as string | undefined,
      relationshipType: row['relationship_type'] as string | undefined,
      organization: row['organization'] as string | undefined,
      role: row['role'] as string | undefined,
      location: row['location'] as string | undefined,
      email: row['email'] as string | undefined,
      phone: row['phone'] as string | undefined,
      socialLinks: safeJsonParse<Record<string, string>>(row['social_links'] as string, {}),
      lastContactAt: row['last_contact_at'] ? new Date(row['last_contact_at'] as string) : undefined,
      contactFrequency: row['contact_frequency'] as number,
      preferredChannel: row['preferred_channel'] as string | undefined,
      sharedTopics: safeJsonParse<string[]>(row['shared_topics'] as string, []),
      sharedProjects: safeJsonParse<string[]>(row['shared_projects'] as string, []),
      notes: row['notes'] as string | undefined,
      relationshipHealth: row['relationship_health'] as number,
      createdAt: new Date(row['created_at'] as string),
      updatedAt: new Date(row['updated_at'] as string),
    };
  }

  /**
   * Validate input for creating or updating a person.
   */
  private validateInput(input: PersonNodeInput | Partial<PersonNodeInput>, isCreate: boolean): void {
    if (isCreate && !input.name) {
      throw new PersonRepositoryError('Name is required', 'NAME_REQUIRED');
    }

    validateStringLength(input.name, MAX_NAME_LENGTH, 'Name');
    validateStringLength(input.notes, MAX_CONTENT_LENGTH, 'Notes');
    validateStringLength(input.howWeMet, MAX_CONTENT_LENGTH, 'How we met');
    validateArrayLength(input.aliases, MAX_ARRAY_COUNT, 'Aliases');
    validateArrayLength(input.sharedTopics, MAX_ARRAY_COUNT, 'Shared topics');
    validateArrayLength(input.sharedProjects, MAX_ARRAY_COUNT, 'Shared projects');
  }

  /**
   * Find a person by their unique ID.
   */
  async findById(id: string): Promise<PersonNode | null> {
    return this.lock.withRead(() => {
      try {
        const stmt = this.db.prepare('SELECT * FROM people WHERE id = ?');
        const row = stmt.get(id) as Record<string, unknown> | undefined;
        if (!row) return null;
        return this.rowToEntity(row);
      } catch (error) {
        throw new PersonRepositoryError(
          `Failed to find person: ${id}`,
          'FIND_BY_ID_FAILED',
          error
        );
      }
    });
  }

  /**
   * Find a person by their name or alias.
   */
  async findByName(name: string): Promise<PersonNode | null> {
    return this.lock.withRead(() => {
      try {
        validateStringLength(name, MAX_NAME_LENGTH, 'Name');

        // Escape special LIKE characters to prevent injection
        const escapedName = name
          .replace(/\\/g, '\\\\')
          .replace(/%/g, '\\%')
          .replace(/_/g, '\\_')
          .replace(/"/g, '\\"');

        const stmt = this.db.prepare(`
          SELECT * FROM people
          WHERE name = ? OR aliases LIKE ? ESCAPE '\\'
        `);
        const row = stmt.get(name, `%"${escapedName}"%`) as Record<string, unknown> | undefined;
        if (!row) return null;
        return this.rowToEntity(row);
      } catch (error) {
        if (error instanceof PersonRepositoryError) throw error;
        throw new PersonRepositoryError(
          'Failed to find person by name',
          'FIND_BY_NAME_FAILED',
          error
        );
      }
    });
  }

  /**
   * Search for people by name (partial match).
   */
  async searchByName(query: string, options: PaginationOptions = {}): Promise<PaginatedResult<PersonNode>> {
    return this.lock.withRead(() => {
      try {
        validateStringLength(query, MAX_NAME_LENGTH, 'Search query');

        const { limit = DEFAULT_LIMIT, offset = 0 } = options;
        const safeLimit = Math.min(Math.max(1, limit), MAX_LIMIT);
        const safeOffset = Math.max(0, offset);

        // Escape special LIKE characters
        const escapedQuery = query
          .replace(/\\/g, '\\\\')
          .replace(/%/g, '\\%')
          .replace(/_/g, '\\_');

        const searchPattern = `%${escapedQuery}%`;

        // Get total count
        const countStmt = this.db.prepare(`
          SELECT COUNT(*) as total FROM people
          WHERE name LIKE ? ESCAPE '\\' OR aliases LIKE ? ESCAPE '\\'
        `);
        const countResult = countStmt.get(searchPattern, searchPattern) as { total: number };
        const total = countResult.total;

        // Get paginated results
        const stmt = this.db.prepare(`
          SELECT * FROM people
          WHERE name LIKE ? ESCAPE '\\' OR aliases LIKE ? ESCAPE '\\'
          ORDER BY name
          LIMIT ? OFFSET ?
        `);
        const rows = stmt.all(searchPattern, searchPattern, safeLimit, safeOffset) as Record<string, unknown>[];
        const items = rows.map(row => this.rowToEntity(row));

        return {
          items,
          total,
          limit: safeLimit,
          offset: safeOffset,
          hasMore: safeOffset + items.length < total,
        };
      } catch (error) {
        if (error instanceof PersonRepositoryError) throw error;
        throw new PersonRepositoryError(
          'Search by name failed',
          'SEARCH_BY_NAME_FAILED',
          error
        );
      }
    });
  }

  /**
   * Create a new person.
   */
  async create(input: PersonNodeInput): Promise<PersonNode> {
    return this.lock.withWrite(() => {
      try {
        this.validateInput(input, true);

        // Validate relationship health is within bounds
        const relationshipHealth = Math.max(0, Math.min(1, input.relationshipHealth ?? 0.5));

        const id = nanoid();
        const now = new Date().toISOString();

        const stmt = this.db.prepare(`
          INSERT INTO people (
            id, name, aliases,
            how_we_met, relationship_type, organization, role, location,
            email, phone, social_links,
            last_contact_at, contact_frequency, preferred_channel,
            shared_topics, shared_projects,
            notes, relationship_health,
            created_at, updated_at
          ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        `);

        stmt.run(
          id,
          input.name,
          JSON.stringify(input.aliases || []),
          input.howWeMet || null,
          input.relationshipType || null,
          input.organization || null,
          input.role || null,
          input.location || null,
          input.email || null,
          input.phone || null,
          JSON.stringify(input.socialLinks || {}),
          input.lastContactAt?.toISOString() || null,
          input.contactFrequency || 0,
          input.preferredChannel || null,
          JSON.stringify(input.sharedTopics || []),
          JSON.stringify(input.sharedProjects || []),
          input.notes || null,
          relationshipHealth,
          now,
          now
        );

        return {
          id,
          name: input.name,
          aliases: input.aliases || [],
          howWeMet: input.howWeMet,
          relationshipType: input.relationshipType,
          organization: input.organization,
          role: input.role,
          location: input.location,
          email: input.email,
          phone: input.phone,
          socialLinks: input.socialLinks || {},
          lastContactAt: input.lastContactAt,
          contactFrequency: input.contactFrequency || 0,
          preferredChannel: input.preferredChannel,
          sharedTopics: input.sharedTopics || [],
          sharedProjects: input.sharedProjects || [],
          notes: input.notes,
          relationshipHealth,
          createdAt: new Date(now),
          updatedAt: new Date(now),
        };
      } catch (error) {
        if (error instanceof PersonRepositoryError) throw error;
        throw new PersonRepositoryError(
          'Failed to create person',
          'CREATE_FAILED',
          error
        );
      }
    });
  }

  /**
   * Update an existing person.
   */
  async update(id: string, updates: Partial<PersonNodeInput>): Promise<PersonNode | null> {
    return this.lock.withWrite(() => {
      try {
        this.validateInput(updates, false);

        // First check if the person exists
        const existingStmt = this.db.prepare('SELECT * FROM people WHERE id = ?');
        const existing = existingStmt.get(id) as Record<string, unknown> | undefined;
        if (!existing) return null;

        const now = new Date().toISOString();

        // Build update statement dynamically based on provided fields
        const setClauses: string[] = ['updated_at = ?'];
        const values: unknown[] = [now];

        if (updates.name !== undefined) {
          setClauses.push('name = ?');
          values.push(updates.name);
        }
        if (updates.aliases !== undefined) {
          setClauses.push('aliases = ?');
          values.push(JSON.stringify(updates.aliases));
        }
        if (updates.howWeMet !== undefined) {
          setClauses.push('how_we_met = ?');
          values.push(updates.howWeMet || null);
        }
        if (updates.relationshipType !== undefined) {
          setClauses.push('relationship_type = ?');
          values.push(updates.relationshipType || null);
        }
        if (updates.organization !== undefined) {
          setClauses.push('organization = ?');
          values.push(updates.organization || null);
        }
        if (updates.role !== undefined) {
          setClauses.push('role = ?');
          values.push(updates.role || null);
        }
        if (updates.location !== undefined) {
          setClauses.push('location = ?');
          values.push(updates.location || null);
        }
        if (updates.email !== undefined) {
          setClauses.push('email = ?');
          values.push(updates.email || null);
        }
        if (updates.phone !== undefined) {
          setClauses.push('phone = ?');
          values.push(updates.phone || null);
        }
        if (updates.socialLinks !== undefined) {
          setClauses.push('social_links = ?');
          values.push(JSON.stringify(updates.socialLinks));
        }
        if (updates.lastContactAt !== undefined) {
          setClauses.push('last_contact_at = ?');
          values.push(updates.lastContactAt?.toISOString() || null);
        }
        if (updates.contactFrequency !== undefined) {
          setClauses.push('contact_frequency = ?');
          values.push(updates.contactFrequency);
        }
        if (updates.preferredChannel !== undefined) {
          setClauses.push('preferred_channel = ?');
          values.push(updates.preferredChannel || null);
        }
        if (updates.sharedTopics !== undefined) {
          setClauses.push('shared_topics = ?');
          values.push(JSON.stringify(updates.sharedTopics));
        }
        if (updates.sharedProjects !== undefined) {
          setClauses.push('shared_projects = ?');
          values.push(JSON.stringify(updates.sharedProjects));
        }
        if (updates.notes !== undefined) {
          setClauses.push('notes = ?');
          values.push(updates.notes || null);
        }
        if (updates.relationshipHealth !== undefined) {
          const health = Math.max(0, Math.min(1, updates.relationshipHealth));
          setClauses.push('relationship_health = ?');
          values.push(health);
        }

        values.push(id);

        const stmt = this.db.prepare(`
          UPDATE people
          SET ${setClauses.join(', ')}
          WHERE id = ?
        `);
        stmt.run(...values);

        // Fetch and return the updated person
        const updatedRow = existingStmt.get(id) as Record<string, unknown>;
        return this.rowToEntity(updatedRow);
      } catch (error) {
        if (error instanceof PersonRepositoryError) throw error;
        throw new PersonRepositoryError(
          `Failed to update person: ${id}`,
          'UPDATE_FAILED',
          error
        );
      }
    });
  }

  /**
   * Delete a person by ID.
   */
  async delete(id: string): Promise<boolean> {
    return this.lock.withWrite(() => {
      try {
        const stmt = this.db.prepare('DELETE FROM people WHERE id = ?');
        const result = stmt.run(id);
        return result.changes > 0;
      } catch (error) {
        throw new PersonRepositoryError(
          `Failed to delete person: ${id}`,
          'DELETE_FAILED',
          error
        );
      }
    });
  }

  /**
   * Get people who haven't been contacted recently.
   */
  async getPeopleToReconnect(
    daysSinceContact: number = 30,
    options: PaginationOptions = {}
  ): Promise<PaginatedResult<PersonNode>> {
    return this.lock.withRead(() => {
      try {
        const { limit = DEFAULT_LIMIT, offset = 0 } = options;
        const safeLimit = Math.min(Math.max(1, limit), MAX_LIMIT);
        const safeOffset = Math.max(0, offset);

        const cutoffDate = new Date();
        cutoffDate.setDate(cutoffDate.getDate() - daysSinceContact);
        const cutoffStr = cutoffDate.toISOString();

        // Get total count
        const countStmt = this.db.prepare(`
          SELECT COUNT(*) as total FROM people
          WHERE last_contact_at IS NOT NULL AND last_contact_at < ?
        `);
        const countResult = countStmt.get(cutoffStr) as { total: number };
        const total = countResult.total;

        // Get paginated results
        const stmt = this.db.prepare(`
          SELECT * FROM people
          WHERE last_contact_at IS NOT NULL
            AND last_contact_at < ?
          ORDER BY last_contact_at ASC
          LIMIT ? OFFSET ?
        `);
        const rows = stmt.all(cutoffStr, safeLimit, safeOffset) as Record<string, unknown>[];
        const items = rows.map(row => this.rowToEntity(row));

        return {
          items,
          total,
          limit: safeLimit,
          offset: safeOffset,
          hasMore: safeOffset + items.length < total,
        };
      } catch (error) {
        throw new PersonRepositoryError(
          'Failed to get people to reconnect',
          'GET_RECONNECT_FAILED',
          error
        );
      }
    });
  }

  /**
   * Record a contact with a person (updates last_contact_at).
   */
  async recordContact(id: string): Promise<void> {
    return this.lock.withWrite(() => {
      try {
        const stmt = this.db.prepare(`
          UPDATE people
          SET last_contact_at = ?, updated_at = ?
          WHERE id = ?
        `);
        const now = new Date().toISOString();
        const result = stmt.run(now, now, id);

        if (result.changes === 0) {
          throw new PersonRepositoryError(
            `Person not found: ${id}`,
            'PERSON_NOT_FOUND'
          );
        }
      } catch (error) {
        if (error instanceof PersonRepositoryError) throw error;
        throw new PersonRepositoryError(
          `Failed to record contact: ${id}`,
          'RECORD_CONTACT_FAILED',
          error
        );
      }
    });
  }

  /**
   * Find people by organization.
   */
  async findByOrganization(
    org: string,
    options: PaginationOptions = {}
  ): Promise<PaginatedResult<PersonNode>> {
    return this.lock.withRead(() => {
      try {
        validateStringLength(org, MAX_NAME_LENGTH, 'Organization');

        const { limit = DEFAULT_LIMIT, offset = 0 } = options;
        const safeLimit = Math.min(Math.max(1, limit), MAX_LIMIT);
        const safeOffset = Math.max(0, offset);

        // Escape special LIKE characters
        const escapedOrg = org
          .replace(/\\/g, '\\\\')
          .replace(/%/g, '\\%')
          .replace(/_/g, '\\_');

        const searchPattern = `%${escapedOrg}%`;

        // Get total count
        const countStmt = this.db.prepare(`
          SELECT COUNT(*) as total FROM people
          WHERE organization LIKE ? ESCAPE '\\'
        `);
        const countResult = countStmt.get(searchPattern) as { total: number };
        const total = countResult.total;

        // Get paginated results
        const stmt = this.db.prepare(`
          SELECT * FROM people
          WHERE organization LIKE ? ESCAPE '\\'
          ORDER BY name
          LIMIT ? OFFSET ?
        `);
        const rows = stmt.all(searchPattern, safeLimit, safeOffset) as Record<string, unknown>[];
        const items = rows.map(row => this.rowToEntity(row));

        return {
          items,
          total,
          limit: safeLimit,
          offset: safeOffset,
          hasMore: safeOffset + items.length < total,
        };
      } catch (error) {
        if (error instanceof PersonRepositoryError) throw error;
        throw new PersonRepositoryError(
          'Failed to find people by organization',
          'FIND_BY_ORG_FAILED',
          error
        );
      }
    });
  }

  /**
   * Find people by shared topic.
   */
  async findBySharedTopic(
    topic: string,
    options: PaginationOptions = {}
  ): Promise<PaginatedResult<PersonNode>> {
    return this.lock.withRead(() => {
      try {
        validateStringLength(topic, MAX_NAME_LENGTH, 'Topic');

        const { limit = DEFAULT_LIMIT, offset = 0 } = options;
        const safeLimit = Math.min(Math.max(1, limit), MAX_LIMIT);
        const safeOffset = Math.max(0, offset);

        // Escape special LIKE characters and quotes for JSON search
        const escapedTopic = topic
          .replace(/\\/g, '\\\\')
          .replace(/%/g, '\\%')
          .replace(/_/g, '\\_')
          .replace(/"/g, '\\"');

        const searchPattern = `%"${escapedTopic}"%`;

        // Get total count
        const countStmt = this.db.prepare(`
          SELECT COUNT(*) as total FROM people
          WHERE shared_topics LIKE ? ESCAPE '\\'
        `);
        const countResult = countStmt.get(searchPattern) as { total: number };
        const total = countResult.total;

        // Get paginated results
        const stmt = this.db.prepare(`
          SELECT * FROM people
          WHERE shared_topics LIKE ? ESCAPE '\\'
          ORDER BY name
          LIMIT ? OFFSET ?
        `);
        const rows = stmt.all(searchPattern, safeLimit, safeOffset) as Record<string, unknown>[];
        const items = rows.map(row => this.rowToEntity(row));

        return {
          items,
          total,
          limit: safeLimit,
          offset: safeOffset,
          hasMore: safeOffset + items.length < total,
        };
      } catch (error) {
        if (error instanceof PersonRepositoryError) throw error;
        throw new PersonRepositoryError(
          'Failed to find people by shared topic',
          'FIND_BY_TOPIC_FAILED',
          error
        );
      }
    });
  }

  /**
   * Get all people with pagination.
   */
  async getAll(options: PaginationOptions = {}): Promise<PaginatedResult<PersonNode>> {
    return this.lock.withRead(() => {
      try {
        const { limit = DEFAULT_LIMIT, offset = 0 } = options;
        const safeLimit = Math.min(Math.max(1, limit), MAX_LIMIT);
        const safeOffset = Math.max(0, offset);

        // Get total count
        const countResult = this.db.prepare('SELECT COUNT(*) as total FROM people').get() as { total: number };
        const total = countResult.total;

        // Get paginated results
        const stmt = this.db.prepare('SELECT * FROM people ORDER BY name LIMIT ? OFFSET ?');
        const rows = stmt.all(safeLimit, safeOffset) as Record<string, unknown>[];
        const items = rows.map(row => this.rowToEntity(row));

        return {
          items,
          total,
          limit: safeLimit,
          offset: safeOffset,
          hasMore: safeOffset + items.length < total,
        };
      } catch (error) {
        throw new PersonRepositoryError(
          'Failed to get all people',
          'GET_ALL_FAILED',
          error
        );
      }
    });
  }
}
