import codegen from "@cosmwasm/ts-codegen";

codegen({
  contracts: [
    { name: "OmniFlixChannel", dir: "../contracts/omniflix-channel/schema" },
  ],
  outPath: "../deployment-scripts/src/types",
  options: {
    bundle: {
      bundleFile: "index.ts",
      scope: "contracts",
    },
    types: {
      enabled: true,
    },
    client: {
      enabled: true,
    },
    reactQuery: {
      enabled: false,
      optionalClient: false,
      version: "v3",
      mutations: false,
      queryKeys: false,
    },
    recoil: {
      enabled: false,
    },
    messageComposer: {
      enabled: true,
    },
  },
})
  .then(() => {
    console.log("Ts codegen success");
  })
