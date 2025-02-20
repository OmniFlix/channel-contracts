import Context from '../helpers/context';
import ChannelHelper from '../helpers/channel.helper';
import { logger } from '../utils/logger';

const deployAndRemoveAsset = async () => {
    let context = new Context();
    await context.initialize();
    let channelHelper = new ChannelHelper();

    // Instantiate new channel contract
    await channelHelper.InstantiateChannelContract(context);

    // Create a new channel
    let channelId = await channelHelper.CreateChannel(context, "creator", "creator");
    logger.log(1, `Channel created with id: ${channelId}`);

    // Publish an off-chain asset
    let assetName = "ValidAsset";
    let assetDescription = "This is a valid off-chain asset";
    let assetMediaUri = "https://example.com/valid-asset.png";
    let assetSource = {
        off_chain: {
            media_uri: assetMediaUri,
            name: assetName,
            description: assetDescription,
        }
    };

    let publishId = await channelHelper.PublishAsset(context, "creator", channelId, assetSource, true);
    logger.log(1, `Asset published with id: ${publishId}`);

    // Admin removes the asset
    await channelHelper.AdminRemoveAssets(context, "admin", [[channelId, publishId]]);
    logger.log(1, `Asset unpublished with id: ${publishId}`);

    // Log transaction and gas details
    logger.log(1, `Channel creation, asset publishing, and removal completed successfully`);
};

deployAndRemoveAsset().then(() => {
    logger.log(1, 'Deployment script executed successfully');
}).catch((error) => {
    logger.error(`Error executing deployment script: ${error}`);
});
