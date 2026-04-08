import type * as qttyTypes from '@siderust/qtty-web';
import type * as tempochTypes from '@siderust/tempoch-web';
import type * as siderustTypes from '@siderust/siderust-web';

export interface SiderustModules {
  qtty: typeof qttyTypes;
  tempoch: typeof tempochTypes;
  siderust: typeof siderustTypes;
}

let loadPromise: Promise<SiderustModules> | null = null;

export function loadSiderust(): Promise<SiderustModules> {
  if (!loadPromise) {
    loadPromise = (async () => {
      const [qtty, tempoch, siderust] = await Promise.all([
        import('@siderust/qtty-web'),
        import('@siderust/tempoch-web'),
        import('@siderust/siderust-web'),
      ]);

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
