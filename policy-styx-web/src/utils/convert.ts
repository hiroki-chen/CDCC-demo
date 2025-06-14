import _ from 'lodash';

/**
 * Converts a snake_case string to camelCase.
 * e.g., 'session_id' -> 'sessionId'
 * @param str The string to convert.
 */
function toCamel(str: string): string {
  return str.replace(/_([a-z])/g, (g) => g[1].toUpperCase());
}

/**
 * Deeply converts all keys in an object or array of objects from snake_case to camelCase.
 * This is a recursive function.
 * @param obj The object or array to process.
 * @returns A new object or array with camelCase keys.
 */
export function keysToCamel(obj: any): any {
  if (_.isArray(obj)) {
    return obj.map(v => keysToCamel(v));
  }

  if (_.isObject(obj) && obj !== null) {
    return _.reduce(
      obj,
      (result, value, key) => {
        const newKey = toCamel(key);
        result[newKey] = keysToCamel(value);
        return result;
      },
      {} as { [key: string]: any }
    );
  }
}
