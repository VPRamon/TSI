/**
 * Tests for typed API error classes.
 */
import { describe, it, expect } from 'vitest';
import {
  ApiRequestError,
  NotFoundError,
  ValidationError,
  NetworkError,
  ServerError,
  RateLimitError,
  isApiRequestError,
  isNotFoundError,
  isValidationError,
  isNetworkError,
  isRetryableError,
  getErrorMessage,
  getErrorTitle,
} from './errors';

describe('ApiRequestError', () => {
  it('creates an error with message and status code', () => {
    const error = new ApiRequestError('Something went wrong', 500, true);

    expect(error.message).toBe('Something went wrong');
    expect(error.statusCode).toBe(500);
    expect(error.isRetryable).toBe(true);
    expect(error.name).toBe('ApiRequestError');
  });

  it('defaults isRetryable to false', () => {
    const error = new ApiRequestError('Error', 400);
    expect(error.isRetryable).toBe(false);
  });
});

describe('NotFoundError', () => {
  it('creates a 404 error with resource info', () => {
    const error = new NotFoundError('Schedule', 123);

    expect(error.message).toBe('Schedule with ID 123 was not found');
    expect(error.statusCode).toBe(404);
    expect(error.resourceType).toBe('Schedule');
    expect(error.resourceId).toBe(123);
    expect(error.isRetryable).toBe(false);
    expect(error.name).toBe('NotFoundError');
  });
});

describe('ValidationError', () => {
  it('creates a 400 error with field info', () => {
    const error = new ValidationError('Invalid format', 'email');

    expect(error.message).toBe('Invalid format');
    expect(error.statusCode).toBe(400);
    expect(error.field).toBe('email');
    expect(error.isRetryable).toBe(false);
    expect(error.name).toBe('ValidationError');
  });

  it('supports details object', () => {
    const details = { email: ['Invalid format', 'Required'] };
    const error = new ValidationError('Validation failed', undefined, details);

    expect(error.details).toEqual(details);
  });
});

describe('NetworkError', () => {
  it('creates a network error with default message', () => {
    const error = new NetworkError();

    expect(error.message).toBe('Unable to connect to the server. Please check your connection.');
    expect(error.statusCode).toBeUndefined();
    expect(error.isRetryable).toBe(true);
    expect(error.name).toBe('NetworkError');
  });

  it('accepts custom message', () => {
    const error = new NetworkError('Custom network error');
    expect(error.message).toBe('Custom network error');
  });
});

describe('ServerError', () => {
  it('creates a 500 error', () => {
    const error = new ServerError();

    expect(error.statusCode).toBe(500);
    expect(error.isRetryable).toBe(true);
    expect(error.name).toBe('ServerError');
  });
});

describe('RateLimitError', () => {
  it('creates a 429 error with retry info', () => {
    const error = new RateLimitError(60);

    expect(error.statusCode).toBe(429);
    expect(error.retryAfter).toBe(60);
    expect(error.isRetryable).toBe(true);
    expect(error.name).toBe('RateLimitError');
  });
});

describe('Type guards', () => {
  it('isApiRequestError returns true for ApiRequestError instances', () => {
    expect(isApiRequestError(new ApiRequestError('test'))).toBe(true);
    expect(isApiRequestError(new NotFoundError('Type', 1))).toBe(true);
    expect(isApiRequestError(new Error('test'))).toBe(false);
    expect(isApiRequestError('string')).toBe(false);
  });

  it('isNotFoundError returns true for NotFoundError instances', () => {
    expect(isNotFoundError(new NotFoundError('Type', 1))).toBe(true);
    expect(isNotFoundError(new ApiRequestError('test'))).toBe(false);
  });

  it('isValidationError returns true for ValidationError instances', () => {
    expect(isValidationError(new ValidationError('test'))).toBe(true);
    expect(isValidationError(new ApiRequestError('test'))).toBe(false);
  });

  it('isNetworkError returns true for NetworkError instances', () => {
    expect(isNetworkError(new NetworkError())).toBe(true);
    expect(isNetworkError(new ApiRequestError('test'))).toBe(false);
  });

  it('isRetryableError returns true for retryable errors', () => {
    expect(isRetryableError(new NetworkError())).toBe(true);
    expect(isRetryableError(new ServerError())).toBe(true);
    expect(isRetryableError(new RateLimitError(60))).toBe(true);
    expect(isRetryableError(new NotFoundError('Type', 1))).toBe(false);
    expect(isRetryableError(new ValidationError('test'))).toBe(false);
  });
});

describe('getErrorMessage', () => {
  it('returns user-friendly message for NotFoundError', () => {
    const error = new NotFoundError('Schedule', 123);
    expect(getErrorMessage(error)).toBe('The requested Schedule could not be found.');
  });

  it('returns field-specific message for ValidationError', () => {
    const error = new ValidationError('Invalid format', 'email');
    expect(getErrorMessage(error)).toBe('Invalid email: Invalid format');
  });

  it('returns generic message for unknown errors', () => {
    expect(getErrorMessage('string')).toBe('An unexpected error occurred.');
  });

  it('returns message for standard Error', () => {
    expect(getErrorMessage(new Error('Test error'))).toBe('Test error');
  });
});

describe('getErrorTitle', () => {
  it('returns appropriate titles for different error types', () => {
    expect(getErrorTitle(new NotFoundError('Type', 1))).toBe('Not Found');
    expect(getErrorTitle(new ValidationError('test'))).toBe('Validation Error');
    expect(getErrorTitle(new NetworkError())).toBe('Connection Error');
    expect(getErrorTitle(new ServerError())).toBe('Server Error');
    expect(getErrorTitle(new Error('test'))).toBe('Error');
  });
});
