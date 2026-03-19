// Lightweight assertion helpers for the E2E WebSocket test suite.
// No external dependencies — plain Node.js.

export class AssertionError extends Error {
  constructor(message) {
    super(message);
    this.name = "AssertionError";
  }
}

/**
 * Assert strict equality between actual and expected values.
 */
export function assertEqual(actual, expected, msg) {
  if (actual !== expected) {
    const detail = msg ? `${msg}: ` : "";
    throw new AssertionError(
      `${detail}expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`
    );
  }
}

/**
 * Assert that a value is truthy.
 */
export function assertOk(value, msg) {
  if (!value) {
    const detail = msg ? `${msg}: ` : "";
    throw new AssertionError(
      `${detail}expected truthy value, got ${JSON.stringify(value)}`
    );
  }
}

/**
 * Assert that an array includes a given item (by strict equality).
 */
export function assertIncludes(arr, item, msg) {
  if (!Array.isArray(arr)) {
    throw new AssertionError(
      `${msg || "assertIncludes"}: expected an array, got ${typeof arr}`
    );
  }
  if (!arr.includes(item)) {
    throw new AssertionError(
      `${msg || "assertIncludes"}: array does not include ${JSON.stringify(item)}`
    );
  }
}

/**
 * Assert that an object matches a partial pattern.
 * Every key in `pattern` must exist in `obj` with a matching value.
 */
export function assertMatch(obj, pattern, msg) {
  if (typeof obj !== "object" || obj === null) {
    throw new AssertionError(
      `${msg || "assertMatch"}: expected an object, got ${typeof obj}`
    );
  }
  for (const [key, expected] of Object.entries(pattern)) {
    const actual = obj[key];
    if (typeof expected === "object" && expected !== null) {
      assertMatch(actual, expected, `${msg || "assertMatch"}.${key}`);
    } else if (actual !== expected) {
      throw new AssertionError(
        `${msg || "assertMatch"}: key "${key}" expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`
      );
    }
  }
}

/**
 * Assert that a synchronous function throws an error.
 */
export function assertThrows(fn, msg) {
  let threw = false;
  try {
    fn();
  } catch {
    threw = true;
  }
  if (!threw) {
    throw new AssertionError(
      `${msg || "assertThrows"}: expected function to throw, but it did not`
    );
  }
}

/**
 * Assert that a value is greater than a threshold.
 */
export function assertGreaterThan(actual, threshold, msg) {
  if (!(actual > threshold)) {
    const detail = msg ? `${msg}: ` : "";
    throw new AssertionError(
      `${detail}expected ${JSON.stringify(actual)} > ${JSON.stringify(threshold)}`
    );
  }
}

/**
 * Assert that an array has the expected length.
 */
export function assertLength(arr, expected, msg) {
  if (!Array.isArray(arr)) {
    throw new AssertionError(
      `${msg || "assertLength"}: expected an array, got ${typeof arr}`
    );
  }
  if (arr.length !== expected) {
    throw new AssertionError(
      `${msg || "assertLength"}: expected length ${expected}, got ${arr.length}`
    );
  }
}

/**
 * Assert that a string or value is of a specific type.
 */
export function assertType(value, expectedType, msg) {
  const actual = typeof value;
  if (actual !== expectedType) {
    const detail = msg ? `${msg}: ` : "";
    throw new AssertionError(
      `${detail}expected type "${expectedType}", got "${actual}"`
    );
  }
}
