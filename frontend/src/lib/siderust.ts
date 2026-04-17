export interface SiderustModules {
  qtty: any;
  tempoch: any;
  siderust: any;
}

export function loadSiderust(): Promise<SiderustModules> {
  return Promise.reject(
    new Error('Siderust WASM modules are unavailable in this frontend build.')
  );
}
