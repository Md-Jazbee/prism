import * as vscode from "vscode";

export interface DecorationController {
  setSliceRanges(ranges: vscode.Range[]): void;
  setHotspots(ranges: vscode.Range[]): void;
  setAmbiguity(ranges: vscode.Range[]): void;
  clear(): void;
  dispose(): void;
}

/**
 * Decoration are off by default (prism.decorations.enabled).
 * Minimal set when enabled: slice highlight + optional ambiguity gutter.
 */
export function createDecorations(
  getEnabled: () => boolean,
): DecorationController {
  const sliceType = vscode.window.createTextEditorDecorationType({
    backgroundColor: new vscode.ThemeColor("editor.findMatchHighlightBackground"),
    isWholeLine: true,
  });
  const hotspotType = vscode.window.createTextEditorDecorationType({
    overviewRulerColor: new vscode.ThemeColor("editorOverviewRuler.warningForeground"),
    overviewRulerLane: vscode.OverviewRulerLane.Right,
  });
  const ambiguityType = vscode.window.createTextEditorDecorationType({
    gutterIconPath: undefined,
    overviewRulerColor: new vscode.ThemeColor("editorOverviewRuler.infoForeground"),
    overviewRulerLane: vscode.OverviewRulerLane.Left,
    light: { after: { contentText: " ?", color: "#888" } },
    dark: { after: { contentText: " ?", color: "#aaa" } },
  });

  let slice: vscode.Range[] = [];
  let hotspots: vscode.Range[] = [];
  let ambiguity: vscode.Range[] = [];

  const paint = () => {
    const ed = vscode.window.activeTextEditor;
    if (!ed || !getEnabled()) {
      ed?.setDecorations(sliceType, []);
      ed?.setDecorations(hotspotType, []);
      ed?.setDecorations(ambiguityType, []);
      return;
    }
    ed.setDecorations(sliceType, slice);
    ed.setDecorations(hotspotType, hotspots);
    ed.setDecorations(ambiguityType, ambiguity);
  };

  const sub = vscode.window.onDidChangeActiveTextEditor(() => paint());

  return {
    setSliceRanges(ranges) {
      slice = ranges;
      paint();
    },
    setHotspots(ranges) {
      hotspots = ranges;
      paint();
    },
    setAmbiguity(ranges) {
      ambiguity = ranges;
      paint();
    },
    clear() {
      slice = [];
      hotspots = [];
      ambiguity = [];
      paint();
    },
    dispose() {
      sub.dispose();
      sliceType.dispose();
      hotspotType.dispose();
      ambiguityType.dispose();
    },
  };
}
