import type { GetNextPageParamFunction, GetPreviousPageParamFunction } from "@tanstack/react-query";

export type PaginateQuery<T> = {
  results: T[];
  count: number;
  next: string | null;
  previous: string | null;
};

type KeyParams = {
  [key: string]: any;
};
export const DEFAULT_LIMIT = 10;

export function getQueryKey<T extends KeyParams>(key: string, params?: T) {
  return [key, ...(params ? [params] : [])];
}

// for infinite query pages  to flatList data
export function normalizePages<T>(pages?: PaginateQuery<T>[]): T[] {
  return pages ? pages.flatMap((page) => page.results) : [];
}

// a function that accept a url and return params as an object
export function getUrlParameters(url: string | null): { [k: string]: string } | null {
  if (url === null) {
    return null;
  }
  const regex = /[?&]([^=#]+)=([^&#]*)/g;
  const params: { [k: string]: string } = {};
  let match = regex.exec(url);
  while (match !== null) {
    if (match[1] !== null) {
      params[match[1]] = match[2];
    }
    match = regex.exec(url);
  }
  return params;
}

export const getPreviousPageParam: GetNextPageParamFunction<unknown, PaginateQuery<unknown>> = (page) =>
  getUrlParameters(page.previous)?.offset ?? null;

export const getNextPageParam: GetPreviousPageParamFunction<unknown, PaginateQuery<unknown>> = (page) =>
  getUrlParameters(page.next)?.offset ?? null;
