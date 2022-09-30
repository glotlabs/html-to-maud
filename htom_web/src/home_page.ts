import init from "../wasm/htom.js";
import { homePage } from "../wasm/htom";
import { Polyester, rustEnum, BrowserWindow } from "polyester";
import { AceEditorElement } from "poly-ace-editor";

// poly-ace-editor is imported to make the custom element available
// Assign to variable to prevent dead code elimination
const _AceEditorElement = AceEditorElement;

(async () => {
  await init("/wasm/htom_bg.wasm");

  const browserWindow = new BrowserWindow();
  const windowSize = browserWindow.getSize();

  const polyester = new Polyester(homePage(windowSize));
  polyester.init();
})();
