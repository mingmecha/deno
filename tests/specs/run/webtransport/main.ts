// Copyright 2018-2024 the Deno authors. All rights reserved. MIT license.

import { decodeBase64 } from "@std/encoding/base64";
import { assertEquals } from "@std/assert";

const cert = Deno.readTextFileSync("../../../testdata/tls/localhost.crt");
const certHash = await crypto.subtle.digest(
  "SHA-256",
  decodeBase64(cert.split("\n").slice(1, -2).join("")),
);

const server = await Deno.listenQuic({
  hostname: "::1",
  port: 4443,
  cert,
  key: Deno.readTextFileSync("../../../testdata/tls/localhost.key"),
  alpnProtocols: ["h3"],
});

(async () => {
  for await (const conn of server) {
    const wt = Deno.upgradeWebTransport(conn);

    wt.ready.then(() => {
      (async () => {
        for await (const bidi of wt.incomingBidirectionalStreams) {
          bidi.readable.pipeTo(bidi.writable).catch(() => {});
        }
      })();

      (async () => {
        for await (const stream of wt.incomingUnidirectionalStreams) {
          const out = await wt.createUnidirectionalStream();
          stream.pipeTo(out).catch(() => {});
        }
      })();

      wt.datagrams.readable.pipeTo(wt.datagrams.writable).catch(() => {});
    });
  }
})();

const client = new WebTransport("https://localhost:4443", {
  serverCertificateHashes: [{
    algorithm: "sha-256",
    value: certHash,
  }],
});

client.ready.then(async () => {
  const bi = await client.createBidirectionalStream();

  {
    const writer = bi.writable.getWriter();
    await writer.write(new Uint8Array([1, 0, 1, 0]));
    writer.releaseLock();
  }

  {
    const reader = bi.readable.getReader();
    assertEquals(await reader.read(), {
      value: new Uint8Array([1, 0, 1, 0]),
      done: false,
    });
    reader.releaseLock();
  }

  {
    const uni = await client.createUnidirectionalStream();
    const writer = uni.getWriter();
    await writer.write(new Uint8Array([0, 2, 0, 2]));
    writer.releaseLock();
  }

  {
    const uni =
      (await client.incomingUnidirectionalStreams.getReader().read()).value;
    const reader = uni.getReader();
    assertEquals(await reader.read(), {
      value: new Uint8Array([0, 2, 0, 2]),
      done: false,
    });
    reader.releaseLock();
  }

  await client.datagrams.writable.getWriter().write(
    new Uint8Array([3, 0, 3, 0]),
  );
  assertEquals(await client.datagrams.readable.getReader().read(), {
    value: new Uint8Array([3, 0, 3, 0]),
    done: false,
  });

  client.close();
  server.close();
});