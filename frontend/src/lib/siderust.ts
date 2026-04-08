import * as qtty from '@siderust/qtty-web';
import * as tempoch from '@siderust/tempoch-web';
import * as siderust from '@siderust/siderust-web';

export interface SiderustModules {
  qtty: typeof qtty;
  tempoch: typeof tempoch;
  siderust: typeof siderust;
}

let loadPromise: Promise<SiderustModules> | null = null;

export function loadSiderust(): Promise<SiderustModules> {
  if (!loadPromise) {
    loadPromise = (async () => {
      await qtty.init();
      await tempoch.init();
      await siderust.init();

      return { qtty, tempoch, siderust };
    })().catch((error) => {
      loadPromise = null;
      throw error;
    });
  }

  return loadPromise;
}
