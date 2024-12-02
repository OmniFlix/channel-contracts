import Context from '../helpers/context';
import ChannelHelper from '../helpers/channel.helper';
import { logger } from '../utils/logger';
import OnftHelper from '../helpers/onft.helper';


const publishNftAsset = async () => {
    let context = new Context;
    await context.initialize();
    let channelHelper = new ChannelHelper;
    let onftHelper = new OnftHelper;
    // Instantiate new channel contract
    await channelHelper.InstantiateChannelContract(context);

    // Create new channel
    let channelId = await channelHelper.CreateChannel(context, "creator", "TestChannel");

    // Create an ONFT collection
    logger.log(1, `Creating ONFT collection with id: ${channelId}`);
    logger.log(1, `This will serve as the collection for the asset being published`);
    let testingCollectionId = "OmniflixTestingCollection" + Math.floor(Math.random() * 10000);
    let collectionId = await onftHelper.createOnftCollection(context, "creator", testingCollectionId);

    // Mint an Asset from the ONFT collection
    // Creator mints an asset.

    logger.log(1, `Minting asset for collection with id: ${collectionId}`);
    logger.log(1, `This asset will be published to the channel created above`);
    let assetId = "OmniflixTestingAsset" + Math.floor(Math.random() * 10000);
    await onftHelper.mintOnft(context, "creator", "creator", collectionId, assetId);

    // Publish the asset to the channel
    logger.log(1, `Publishing asset with id: ${assetId} to channel with id: ${channelId}`);
    await channelHelper.PublishAsset(context, "creator", channelId, collectionId, assetId, true);

    // Query Assets 
    logger.log(1, `Querying assets for channel with id: ${channelId}`);
    let assets = await channelHelper.QueryAssets(context, channelId);
    logger.log(1, `Assets for channel with id: ${channelId}: ${JSON.stringify(assets)}`);
}

const publishOffChainAsset = async () => {
    let context = new Context;
    await context.initialize();
    let channelHelper = new ChannelHelper;
    let onftHelper = new OnftHelper;
    // Instantiate new channel contract
    await channelHelper.InstantiateChannelContract(context);

    // Create new channel
    let channelId = await channelHelper.CreateChannel(context, "creator", "TestChannel");

    // Publish the asset to the channel
    logger.log(1, `Publishing off-chain asset to channel with id: ${channelId}`);
    let assetName = "OmniflixTestingAsset" + Math.floor(Math.random() * 10000);
    let assetDescription = "This is a test asset";
    let assetMediaUri = "https://ipfs.io/OmniFlixTestingAsset";
    await channelHelper.PublishOffchainAsset(context, "creator", channelId, assetMediaUri, assetName, assetDescription, false);

    // Query Assets 
    logger.log(1, `Querying assets for channel with id: ${channelId}`);
    let assets = await channelHelper.QueryAssets(context, channelId);
    logger.log(1, `Assets for channel with id: ${channelId}: ${JSON.stringify(assets)}`);
}


publishNftAsset().then(() => {
    publishOffChainAsset().then(() => {
        logger.log(1, `Asset publishing completed successfully`);
    });
}
);

