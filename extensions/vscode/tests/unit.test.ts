import { describe, expect, it } from "vitest";
import { PrismSession } from "../src/session";
import { PrismApiError } from "../src/types";
import { redactPackForLlm, generateAgentsMd, registerMcp } from "../src/agent/assets";
import { __test } from "../src/transport/client";
import { resolvePrismBinary, downloadOffer } from "../src/lifecycle/binary";
import * as fs from "node:fs";
import * as os from "node:os";
import * as path from "node:path";

describe("session citation peek", () => {
  it("resolves C1 from citations", () => {
    const s = new PrismSession();
    s.setPack({
      meta: {
        intent: "repo_qa",
        budget_tokens: 100,
        tokens_used: 10,
        question: "q",
      },
      hierarchy: {},
      fragments: [
        {
          id: "frag:a",
          kind: "slice",
          layer: "core",
          text: "hello",
          token_estimate: 1,
          provenance: { node_ids: ["sym:Helper"] },
        },
      ],
      citations: [{ id: "C1", fragment_id: "frag:a", node_ids: ["sym:Helper"] }],
    });
    expect(s.citationById("C1")?.nodeIds?.[0]).toBe("sym:Helper");
    expect(s.citationById("1")?.fragmentId).toBe("frag:a");
  });
});

describe("transport version major", () => {
  it("parses major", () => {
    expect(__test.majorOf("0.0.1")).toBe(0);
    expect(__test.majorOf("1.2.3")).toBe(1);
  });
});

describe("compile unwrap", () => {
  it("maps scope_unresolved to PrismApiError", () => {
    expect(() =>
      __test.unwrapCompile({
        status: "scope_unresolved",
        data: { code: "SCOPE_UNRESOLVED", message: "need anchor" },
      }),
    ).toThrow(PrismApiError);
  });

  it("unwraps ok pack", () => {
    const pack = __test.unwrapCompile({
      status: "ok",
      data: {
        meta: { intent: "repo_qa", budget_tokens: 10, tokens_used: 1, question: "q" },
        hierarchy: {},
        fragments: [],
      },
    });
    expect(pack.meta.intent).toBe("repo_qa");
  });
});

describe("redact pack", () => {
  it("truncates long fragment text", () => {
    const out = JSON.parse(
      redactPackForLlm({
        meta: { intent: "x" },
        fragments: [{ id: "1", kind: "k", layer: "l", text: "a".repeat(2000) }],
      }),
    );
    expect(out.fragments[0].text.length).toBeLessThan(900);
    expect(out.fragments[0].text).toContain("redacted");
  });
});

describe("refusal error shape", () => {
  it("carries code", () => {
    const e = new PrismApiError({
      code: "SCOPE_UNRESOLVED",
      message: "need anchor",
      hint: "pick symbol",
    });
    expect(e.code).toBe("SCOPE_UNRESOLVED");
  });
});

describe("binary resolve", () => {
  it("honors explicit binaryPath setting", () => {
    const tmp = fs.mkdtempSync(path.join(os.tmpdir(), "prism-ext-"));
    const fake = path.join(tmp, "fake-prism");
    fs.writeFileSync(fake, "#!/bin/sh\n");
    fs.chmodSync(fake, 0o755);
    const hit = resolvePrismBinary(tmp, fake);
    expect(hit?.source).toBe("setting");
    expect(hit?.prismPath).toBe(fake);
  });

  it("download offer disabled without base url", () => {
    const offer = downloadOffer("", "/no/such/manifest.json");
    expect(offer.enabled).toBe(false);
  });
});

describe("agent assets", () => {
  it("writes AGENTS.md and registers mcp.json", async () => {
    const tmp = fs.mkdtempSync(path.join(os.tmpdir(), "prism-agent-"));
    fs.mkdirSync(path.join(tmp, ".cursor"), { recursive: true });
    const agents = await generateAgentsMd(tmp);
    expect(fs.existsSync(agents)).toBe(true);
    expect(fs.readFileSync(agents, "utf8")).toContain("compile_context");
    const msg = await registerMcp(tmp, "/usr/bin/prism", true);
    expect(msg).toContain("Registered");
    const mcp = JSON.parse(
      fs.readFileSync(path.join(tmp, ".cursor", "mcp.json"), "utf8"),
    );
    expect(mcp.mcpServers.prism.command).toBe("/usr/bin/prism");
    await registerMcp(tmp, "/usr/bin/prism", false);
    const mcp2 = JSON.parse(
      fs.readFileSync(path.join(tmp, ".cursor", "mcp.json"), "utf8"),
    );
    expect(mcp2.mcpServers.prism).toBeUndefined();
  });
});

describe("activation contract", () => {
  it("activate module exports activate/deactivate", async () => {
    const src = fs.readFileSync(
      path.join(__dirname, "../src/extension.ts"),
      "utf8",
    );
    expect(src).toContain("intentionally thin");
    expect(src).toContain("export function activate");
  });
});
