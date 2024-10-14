// This file is used as a template for the deployment_config.ts file that is used to deploy streamswap-v2 //
// The deployment_config.ts file should be in the following format //

export const deploymentConfig = {
    // CHANNEL CONTRACT CONFIGURATION //
    channel_creation_fee: "1000000", // The fee required to create a channel
    channels_collection_id: "Channels-test", // The collection id of the channels
    channels_collection_name: "OmniFlix Channels", // The name of the channels collection
    channels_collection_symbol: "OFC", // The symbol of the channels collection


    artifacts_path: "../artifacts" // The path to the artifacts
};
