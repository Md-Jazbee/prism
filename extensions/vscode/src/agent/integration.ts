import * as vscode from "vscode";
import { PrismApiError } from "../types";

export {
  registerMcp,
  generateAgentsMd,
  redactPackForLlm,
} from "./assets";

/** Map API refusals to actionable QuickPick / messages. */
export async function handleRefusal(err: PrismApiError): Promise<void> {
  switch (err.code) {
    case "SCOPE_UNRESOLVED": {
      const pick = await vscode.window.showWarningMessage(
        `Scope unresolved: ${err.message}`,
        "Pick Anchor",
        "Dismiss",
      );
      if (pick === "Pick Anchor") {
        await vscode.commands.executeCommand("prism.pickAnchor");
      }
      break;
    }
    case "PRECISION_REQUIRED": {
      const pick = await vscode.window.showWarningMessage(
        `Precision required: ${err.message}`,
        "Open SCIP Runbook",
        "Dismiss",
      );
      if (pick === "Open SCIP Runbook") {
        const folder = vscode.workspace.workspaceFolders?.[0]?.uri;
        if (folder) {
          const doc = vscode.Uri.joinPath(
            folder,
            "docs",
            "architecture",
            "SCIP-RUNBOOK.md",
          );
          try {
            await vscode.window.showTextDocument(doc);
          } catch {
            void vscode.window.showInformationMessage(
              err.hint ?? "Run: prism precise import (see SCIP-RUNBOOK)",
            );
          }
        }
      }
      break;
    }
    case "INDEX_UNAVAILABLE": {
      const pick = await vscode.window.showWarningMessage(
        "Index unavailable",
        "Build Index",
        "Dismiss",
      );
      if (pick === "Build Index") {
        await vscode.commands.executeCommand("prism.buildIndex");
      }
      break;
    }
    case "VIEW_TOO_LARGE": {
      const anchors = err.suggestedAnchors?.join(", ");
      void vscode.window.showWarningMessage(
        `View too large. ${err.hint ?? ""} ${anchors ? `Try anchors: ${anchors}` : ""}`.trim(),
      );
      break;
    }
    case "VERSION_SKEW": {
      void vscode.window.showErrorMessage(
        `${err.message}. ${err.hint ?? ""}`.trim(),
      );
      break;
    }
    default:
      void vscode.window.showErrorMessage(`${err.code}: ${err.message}`);
  }
}
