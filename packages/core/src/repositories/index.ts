// Re-export from NodeRepository (primary source for common types)
export {
  NodeRepository,
  type INodeRepository,
  type PaginationOptions,
  type PaginatedResult,
  type GitContext,
} from './NodeRepository.js';

// Re-export from PersonRepository (exclude duplicate types)
export {
  PersonRepository,
  type IPersonRepository,
  type PersonNodeInput,
  PersonRepositoryError,
} from './PersonRepository.js';

// Re-export from EdgeRepository (exclude duplicate types)
export {
  EdgeRepository,
  type IEdgeRepository,
  type GraphEdgeInput,
  type EdgeType,
  type TransitivePath,
  EdgeRepositoryError,
} from './EdgeRepository.js';
