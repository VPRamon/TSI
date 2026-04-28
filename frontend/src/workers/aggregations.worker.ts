/**
 * Web Worker entry point that exposes the pure aggregation helpers from
 * `./aggregations.ts` over a Comlink RPC channel.
 *
 * The components consume this through `aggregationsClient.ts`; this file
 * itself never touches the DOM.
 */
import { expose } from 'comlink';
import {
  binHeatmap,
  computeBlockStatusMap,
  computeParetoFront,
  sortFilterBlocks,
} from './aggregations';

export const aggregationsApi = {
  computeBlockStatusMap,
  sortFilterBlocks,
  binHeatmap,
  computeParetoFront,
};

export type AggregationsApi = typeof aggregationsApi;

expose(aggregationsApi);
