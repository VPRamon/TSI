/**
 * Typed error classes for better error discrimination.
 * Enables granular error handling in components.
 *
 * Note: Named ApiRequestError to avoid conflict with the ApiError interface
 * in types.ts which represents the server's error response shape.
 */

/**
 * Base API request error class.
 */
export class ApiRequestError extends Error {
  public readonly statusCode?: number;
  public readonly isRetryable: boolean;

  constructor(message: string, statusCode?: number, isRetryable = false) {
    super(message);
    this.name = 'ApiRequestError';
    this.statusCode = statusCode;
    this.isRetryable = isRetryable;
    Object.setPrototypeOf(this, ApiRequestError.prototype);
  }
}

/**
 * Resource not found (404).
 */
export class NotFoundError extends ApiRequestError {
  public readonly resourceType: string;
  public readonly resourceId: string | number;

  constructor(resourceType: string, resourceId: string | number) {
    super(`${resourceType} with ID ${resourceId} was not found`, 404, false);
    this.name = 'NotFoundError';
    this.resourceType = resourceType;
    this.resourceId = resourceId;
    Object.setPrototypeOf(this, NotFoundError.prototype);
  }
}

/**
 * Validation error (400).
 */
export class ValidationError extends ApiRequestError {
  public readonly field?: string;
  public readonly details?: Record<string, string[]>;

  constructor(message: string, field?: string, details?: Record<string, string[]>) {
    super(message, 400, false);
    this.name = 'ValidationError';
    this.field = field;
    this.details = details;
    Object.setPrototypeOf(this, ValidationError.prototype);
  }
}

/**
 * Network/connection error.
 */
export class NetworkError extends ApiRequestError {
  constructor(message = 'Unable to connect to the server. Please check your connection.') {
    super(message, undefined, true);
    this.name = 'NetworkError';
    Object.setPrototypeOf(this, NetworkError.prototype);
  }
}

/**
 * Server error (5xx).
 */
export class ServerError extends ApiRequestError {
  constructor(message = 'An unexpected server error occurred. Please try again later.') {
    super(message, 500, true);
    this.name = 'ServerError';
    Object.setPrototypeOf(this, ServerError.prototype);
  }
}

/**
 * Rate limit exceeded (429).
 */
export class RateLimitError extends ApiRequestError {
  public readonly retryAfter?: number;

  constructor(retryAfter?: number) {
    super('Too many requests. Please wait before trying again.', 429, true);
    this.name = 'RateLimitError';
    this.retryAfter = retryAfter;
    Object.setPrototypeOf(this, RateLimitError.prototype);
  }
}

/**
 * Type guard to check if an error is a specific API error type.
 */
export function isApiRequestError(error: unknown): error is ApiRequestError {
  return error instanceof ApiRequestError;
}

export function isNotFoundError(error: unknown): error is NotFoundError {
  return error instanceof NotFoundError;
}

export function isValidationError(error: unknown): error is ValidationError {
  return error instanceof ValidationError;
}

export function isNetworkError(error: unknown): error is NetworkError {
  return error instanceof NetworkError;
}

export function isRetryableError(error: unknown): boolean {
  return isApiRequestError(error) && error.isRetryable;
}

/**
 * Get user-friendly error message.
 */
export function getErrorMessage(error: unknown): string {
  if (isNotFoundError(error)) {
    return `The requested ${error.resourceType} could not be found.`;
  }
  if (isValidationError(error)) {
    return error.field ? `Invalid ${error.field}: ${error.message}` : error.message;
  }
  if (isNetworkError(error)) {
    return error.message;
  }
  if (isApiRequestError(error)) {
    return error.message;
  }
  if (error instanceof Error) {
    return error.message;
  }
  return 'An unexpected error occurred.';
}

/**
 * Get error title for display.
 */
export function getErrorTitle(error: unknown): string {
  if (isNotFoundError(error)) {
    return 'Not Found';
  }
  if (isValidationError(error)) {
    return 'Validation Error';
  }
  if (isNetworkError(error)) {
    return 'Connection Error';
  }
  if (isApiRequestError(error) && error.statusCode && error.statusCode >= 500) {
    return 'Server Error';
  }
  return 'Error';
}
