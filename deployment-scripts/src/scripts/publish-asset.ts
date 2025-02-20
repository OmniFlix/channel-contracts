import Context from '../helpers/context';
import ChannelHelper from '../helpers/channel.helper';
import { logger } from '../utils/logger';
import OnftHelper from '../helpers/onft.helper';
import { AssetSource } from '../types/OmniFlixChannel.types.ts';

const publishNftAsset = async () => {
    let context = new Context;
    await context.initialize();
    let channelHelper = new ChannelHelper;
    let onftHelper = new OnftHelper;
    // Instantiate new channel contract
    await channelHelper.InstantiateChannelContract(context);

    // Create new channel
    let channelId = await channelHelper.CreateChannel(context, "creator", "channel");

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
    await channelHelper.PublishAsset(context, "creator", channelId, {
        nft: {
            collection_id: collectionId,
            onft_id: assetId
        }
    }, true);

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
    let channelId = await channelHelper.CreateChannel(context, "creator", "channel");

    // Publish the asset to the channel
    logger.log(1, `Publishing off-chain asset to channel with id: ${channelId}`);
    let assetName = "OmniflixTestingAsset";
    let assetDescription = "This is a test asset";
    let assetMediaUri = "https://ipfs.io/OmniFlixTestingAsset";
    let assetSource = {
        off_chain: {
            media_uri: assetMediaUri,
            name: assetName,
            description: assetDescription,
        }
    }
    await channelHelper.PublishAsset(context, "creator", channelId, assetSource, false);

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

