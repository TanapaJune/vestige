/**
 * Safe JSON utilities for database operations
 */

import { z } from 'zod';
import { logger } from './logger.js';

/**
 * Safely parse JSON with logging on failure
 */
export function safeJsonParse<T>(
  value: string | null | undefined,
  fallback: T,
  options?: {
    logOnError?: boolean;
    context?: string;
  }
): T {
  if (!value) return fallback;

  try {
    const parsed = JSON.parse(value);

    // Type validation
    if (typeof parsed !== typeof fallback) {
      if (options?.logOnError !== false) {
        logger.warn('JSON parse type mismatch', {
          expected: typeof fallback,
          got: typeof parsed,
          context: options?.context,
        });
      }
      return fallback;
    }

    // Array validation
    if (Array.isArray(fallback) && !Array.isArray(parsed)) {
      if (options?.logOnError !== false) {
        logger.warn('JSON parse expected array', {
          got: typeof parsed,
          context: options?.context,
        });
      }
      return fallback;
    }

    return parsed as T;
  } catch (error) {
    if (options?.logOnError !== false) {
      logger.warn('JSON parse failed', {
        error: (error as Error).message,
        valuePreview: value.slice(0, 100),
        context: options?.context,
      });
    }
    return fallback;
  }
}

/**
 * Safely stringify JSON with circular reference handling
 */
export function safeJsonStringify(
  value: unknown,
  options?: {
    replacer?: (key: string, value: unknown) => unknown;
    space?: number;
    maxDepth?: number;
  }
): string {
  const seen = new WeakSet();
  const maxDepth = options?.maxDepth ?? 10;

  function replacer(
    this: unknown,
    key: string,
    value: unknown,
    depth: number
  ): unknown {
    if (depth > maxDepth) {
      return '[Max Depth Exceeded]';
    }

    if (typeof value === 'object' && value !== null) {
      if (seen.has(value)) {
        return '[Circular Reference]';
      }
      seen.add(value);
    }

    if (options?.replacer) {
      return options.replacer(key, value);
    }

    // Handle special types
    if (value instanceof Error) {
      return {
        name: value.name,
        message: value.message,
        stack: value.stack,
      };
    }

    if (value instanceof Date) {
      return value.toISOString();
    }

    if (value instanceof Map) {
      return Object.fromEntries(value);
    }

    if (value instanceof Set) {
      return Array.from(value);
    }

    return value;
  }

  try {
    // Create a depth-tracking replacer
    let currentDepth = 0;
    return JSON.stringify(
      value,
      function (key, val) {
        if (key === '') currentDepth = 0;
        else currentDepth++;
        return replacer.call(this, key, val, currentDepth);
      },
      options?.space
    );
  } catch (error) {
    logger.error('JSON stringify failed', error as Error);
    return '{}';
  }
}

/**
 * Parse JSON and validate against Zod schema
 */
export function parseJsonWithSchema<T extends z.ZodType>(
  value: string | null | undefined,
  schema: T,
  fallback: z.infer<T>
): z.infer<T> {
  if (!value) return fallback;

  try {
    const parsed = JSON.parse(value);
    const result = schema.safeParse(parsed);

    if (result.success) {
      return result.data;
    }

    logger.warn('JSON schema validation failed', {
      errors: result.error.errors,
    });
    return fallback;
  } catch (error) {
    logger.warn('JSON parse failed for schema validation', {
      error: (error as Error).message,
    });
    return fallback;
  }
}

/**
 * Calculate diff between two JSON objects
 */
export function jsonDiff(
  before: Record<string, unknown>,
  after: Record<string, unknown>
): { added: string[]; removed: string[]; changed: string[] } {
  const added: string[] = [];
  const removed: string[] = [];
  const changed: string[] = [];

  // Check for added and changed
  for (const key of Object.keys(after)) {
    if (!(key in before)) {
      added.push(key);
    } else if (JSON.stringify(before[key]) !== JSON.stringify(after[key])) {
      changed.push(key);
    }
  }

  // Check for removed
  for (const key of Object.keys(before)) {
    if (!(key in after)) {
      removed.push(key);
    }
  }

  return { added, removed, changed };
}

/**
 * Deep merge JSON objects
 */
export function jsonMerge<T extends Record<string, unknown>>(
  target: T,
  ...sources: Partial<T>[]
): T {
  const result = { ...target };

  for (const source of sources) {
    for (const key of Object.keys(source)) {
      const targetVal = result[key as keyof T];
      const sourceVal = source[key as keyof T];

      if (
        typeof targetVal === 'object' &&
        targetVal !== null &&
        typeof sourceVal === 'object' &&
        sourceVal !== null &&
        !Array.isArray(targetVal) &&
        !Array.isArray(sourceVal)
      ) {
        (result as Record<string, unknown>)[key] = jsonMerge(
          targetVal as Record<string, unknown>,
          sourceVal as Record<string, unknown>
        );
      } else if (sourceVal !== undefined) {
        (result as Record<string, unknown>)[key] = sourceVal;
      }
    }
  }

  return result;
}
