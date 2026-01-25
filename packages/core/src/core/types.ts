import { z } from 'zod';

// ============================================================================
// SOURCE TYPES
// ============================================================================

export const SourceTypeSchema = z.enum([
  'note',
  'conversation',
  'email',
  'book',
  'article',
  'highlight',
  'meeting',
  'manual',
  'webpage',
]);
export type SourceType = z.infer<typeof SourceTypeSchema>;

export const SourcePlatformSchema = z.enum([
  'obsidian',
  'notion',
  'roam',
  'logseq',
  'claude',
  'chatgpt',
  'gmail',
  'outlook',
  'kindle',
  'readwise',
  'pocket',
  'instapaper',
  'manual',
  'browser',
]);
export type SourcePlatform = z.infer<typeof SourcePlatformSchema>;

// ============================================================================
// KNOWLEDGE NODE
// ============================================================================

export const KnowledgeNodeSchema = z.object({
  id: z.string(),
  content: z.string(),
  summary: z.string().optional(),

  // Temporal metadata
  createdAt: z.date(),
  updatedAt: z.date(),
  lastAccessedAt: z.date(),
  accessCount: z.number().default(0),

  // Decay modeling (SM-2 inspired spaced repetition)
  retentionStrength: z.number().min(0).max(1).default(1),
  stabilityFactor: z.number().min(1).optional().default(1), // Grows with reviews, flattens decay curve
  sentimentIntensity: z.number().min(0).max(1).optional().default(0), // Emotional weight - higher = decays slower
  nextReviewDate: z.date().optional(),
  reviewCount: z.number().default(0),

  // Dual-Strength Memory Model (Bjork & Bjork, 1992)
  storageStrength: z.number().min(1).default(1),      // How well encoded (never decreases)
  retrievalStrength: z.number().min(0).max(1).default(1), // How accessible now (decays)

  // Provenance
  sourceType: SourceTypeSchema,
  sourcePlatform: SourcePlatformSchema,
  sourceId: z.string().optional(), // Original source reference
  sourceUrl: z.string().optional(),
  sourceChain: z.array(z.string()).default([]), // Full provenance path

  // Git-Blame for Thoughts - what code was being worked on when this memory was created?
  gitContext: z.object({
    branch: z.string().optional(),
    commit: z.string().optional(),        // Short SHA
    commitMessage: z.string().optional(),  // First line of commit message
    repoPath: z.string().optional(),       // Repository root path
    dirty: z.boolean().optional(),         // Had uncommitted changes?
    changedFiles: z.array(z.string()).optional(), // Files with uncommitted changes
  }).optional(),

  // Confidence & quality
  confidence: z.number().min(0).max(1).default(0.8),
  isContradicted: z.boolean().default(false),
  contradictionIds: z.array(z.string()).default([]),

  // Extracted entities
  people: z.array(z.string()).default([]),
  concepts: z.array(z.string()).default([]),
  events: z.array(z.string()).default([]),
  tags: z.array(z.string()).default([]),
});
export type KnowledgeNode = z.infer<typeof KnowledgeNodeSchema>;
// Input type where optional/default fields are truly optional (for insertNode)
export type KnowledgeNodeInput = z.input<typeof KnowledgeNodeSchema>;

// ============================================================================
// PERSON NODE (People Memory / Mini-CRM)
// ============================================================================

export const InteractionTypeSchema = z.enum([
  'meeting',
  'email',
  'call',
  'message',
  'social',
  'collaboration',
  'mention', // Referenced in notes but not direct interaction
]);
export type InteractionType = z.infer<typeof InteractionTypeSchema>;

export const InteractionSchema = z.object({
  id: z.string(),
  personId: z.string(),
  type: InteractionTypeSchema,
  date: z.date(),
  summary: z.string(),
  topics: z.array(z.string()).default([]),
  sentiment: z.number().min(-1).max(1).optional(), // -1 negative, 0 neutral, 1 positive
  actionItems: z.array(z.string()).default([]),
  sourceNodeId: z.string().optional(), // Link to knowledge node if derived
});
export type Interaction = z.infer<typeof InteractionSchema>;

export const PersonNodeSchema = z.object({
  id: z.string(),
  name: z.string(),
  aliases: z.array(z.string()).default([]),

  // Relationship context
  howWeMet: z.string().optional(),
  relationshipType: z.string().optional(), // colleague, friend, mentor, family, etc.
  organization: z.string().optional(),
  role: z.string().optional(),
  location: z.string().optional(),

  // Contact info
  email: z.string().optional(),
  phone: z.string().optional(),
  socialLinks: z.record(z.string()).default({}),

  // Communication patterns
  lastContactAt: z.date().optional(),
  contactFrequency: z.number().default(0), // Interactions per month (calculated)
  preferredChannel: z.string().optional(),

  // Shared context
  sharedTopics: z.array(z.string()).default([]),
  sharedProjects: z.array(z.string()).default([]),

  // Meta
  notes: z.string().optional(),
  relationshipHealth: z.number().min(0).max(1).default(0.5), // Calculated from recency + frequency

  createdAt: z.date(),
  updatedAt: z.date(),
});
export type PersonNode = z.infer<typeof PersonNodeSchema>;

// ============================================================================
// GRAPH EDGES (Relationships)
// ============================================================================

export const EdgeTypeSchema = z.enum([
  'relates_to',
  'derived_from',
  'contradicts',
  'supports',
  'references',
  'part_of',
  'follows', // Temporal sequence
  'person_mentioned',
  'concept_instance',
  'similar_to',
]);
export type EdgeType = z.infer<typeof EdgeTypeSchema>;

export const GraphEdgeSchema = z.object({
  id: z.string(),
  fromId: z.string(),
  toId: z.string(),
  edgeType: EdgeTypeSchema,
  weight: z.number().min(0).max(1).default(0.5),
  metadata: z.record(z.unknown()).default({}),
  createdAt: z.date(),
});
export type GraphEdge = z.infer<typeof GraphEdgeSchema>;

// ============================================================================
// SOURCE TRACKING
// ============================================================================

export const SourceSchema = z.object({
  id: z.string(),
  type: SourceTypeSchema,
  platform: SourcePlatformSchema,
  originalId: z.string().optional(),
  url: z.string().optional(),
  filePath: z.string().optional(),
  title: z.string().optional(),
  author: z.string().optional(),
  publicationDate: z.date().optional(),

  // Sync tracking
  ingestedAt: z.date(),
  lastSyncedAt: z.date(),
  contentHash: z.string().optional(), // For change detection

  // Stats
  nodeCount: z.number().default(0),
});
export type Source = z.infer<typeof SourceSchema>;

// ============================================================================
// TOOL INPUT/OUTPUT SCHEMAS
// ============================================================================

export const IngestInputSchema = z.object({
  content: z.string(),
  source: SourceTypeSchema.optional().default('manual'),
  platform: SourcePlatformSchema.optional().default('manual'),
  sourceId: z.string().optional(),
  sourceUrl: z.string().optional(),
  timestamp: z.string().datetime().optional(),
  people: z.array(z.string()).optional(),
  tags: z.array(z.string()).optional(),
  title: z.string().optional(),
});
export type IngestInput = z.infer<typeof IngestInputSchema>;

export const RecallOptionsSchema = z.object({
  query: z.string(),
  timeRange: z.object({
    start: z.string().datetime().optional(),
    end: z.string().datetime().optional(),
  }).optional(),
  sources: z.array(SourceTypeSchema).optional(),
  platforms: z.array(SourcePlatformSchema).optional(),
  people: z.array(z.string()).optional(),
  minConfidence: z.number().min(0).max(1).optional(),
  limit: z.number().min(1).max(100).optional().default(10),
  includeContext: z.boolean().optional().default(true),
});
export type RecallOptions = z.infer<typeof RecallOptionsSchema>;

export const RecallResultSchema = z.object({
  node: KnowledgeNodeSchema,
  score: z.number(),
  matchType: z.enum(['semantic', 'keyword', 'graph']),
  context: z.string().optional(),
  relatedNodes: z.array(z.string()).optional(),
});
export type RecallResult = z.infer<typeof RecallResultSchema>;

export const SynthesisOptionsSchema = z.object({
  topic: z.string(),
  depth: z.enum(['shallow', 'deep']).optional().default('shallow'),
  format: z.enum(['summary', 'outline', 'narrative']).optional().default('summary'),
  maxSources: z.number().optional().default(20),
});
export type SynthesisOptions = z.infer<typeof SynthesisOptionsSchema>;

// ============================================================================
// DECAY MODELING
// ============================================================================

export interface DecayConfig {
  // Ebbinghaus forgetting curve parameters
  initialRetention: number; // Starting retention (default 1.0)
  decayRate: number; // Base decay rate (default ~0.9 for typical forgetting)
  minRetention: number; // Floor retention (default 0.1)
  reviewBoost: number; // How much review increases retention (default 0.3)
  accessBoost: number; // How much access slows decay (default 0.1)
}

export const DEFAULT_DECAY_CONFIG: DecayConfig = {
  initialRetention: 1.0,
  decayRate: 0.9,
  minRetention: 0.1,
  reviewBoost: 0.3,
  accessBoost: 0.1,
};

// ============================================================================
// DAILY BRIEF
// ============================================================================

export const DailyBriefSchema = z.object({
  date: z.date(),
  stats: z.object({
    totalNodes: z.number(),
    addedToday: z.number(),
    addedThisWeek: z.number(),
    connectionsDiscovered: z.number(),
  }),
  reviewDue: z.array(z.object({
    nodeId: z.string(),
    summary: z.string(),
    lastAccessed: z.date(),
    retentionStrength: z.number(),
  })),
  peopleToReconnect: z.array(z.object({
    personId: z.string(),
    name: z.string(),
    daysSinceContact: z.number(),
    sharedTopics: z.array(z.string()),
  })),
  interestingConnections: z.array(z.object({
    nodeA: z.string(),
    nodeB: z.string(),
    connectionReason: z.string(),
  })),
  recentThemes: z.array(z.string()),
});
export type DailyBrief = z.infer<typeof DailyBriefSchema>;
