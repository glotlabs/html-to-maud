import init from "../wasm/htom.js";
import { homePage } from "../wasm/htom";
import { Polyester, rustEnum, BrowserWindow } from "polyester";

(async () => {
  await init("./wasm/htom_bg.wasm");

  const browserWindow = new BrowserWindow();
  const windowSize = browserWindow.getSize();

  const polyester = new Polyester(homePage(windowSize));

  polyester.init();

  const editor = initAce("html-input");

  editor.getSession().on("change", () => {
    const msg = rustEnum.tuple("HtmlChanged", [editor.getValue()]);
    polyester.send(msg);
  });

  polyester.onAppEffect((effect) => {
    switch (effect.type) {
      case "setKeyboardHandler":
        editor.setKeyboardHandler(effect.config);
        break;
    }
  });
})();

function initAce(elemId: string): any {
  // @ts-ignore
  const editor = ace.edit(elemId);
  // @ts-ignore
  const Mode = ace.require("ace/mode/html").Mode;

  editor.session.setMode(new Mode());
  editor.setShowPrintMargin(false);
  editor.renderer.setShowGutter(false);

  return editor;
}
