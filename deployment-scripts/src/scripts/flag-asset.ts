import Context from '../helpers/context';
import ChannelHelper from '../helpers/channel.helper';
import { logger } from '../utils/logger';
import { Flag } from '../types/OmniFlixChannel.types';

const testFlaggingLogic = async () => {
    let context = new Context();
    await context.initialize();


    let channelHelper = new ChannelHelper();
    await channelHelper.InstantiateChannelContract(context);

    // Step 1: Create a new channel
    let channelId = await channelHelper.CreateChannel(context, "creator", "testboi");
    logger.log(1, `Channel created with id: ${channelId}`);

    // Step 2: Publish an asset
    let assetSource = {
        off_chain: {
            media_uri: "https://example.com/test-asset.png",
            name: "Test Asset",
            description: "This is a test asset",
        }
    };
    let publishId = await channelHelper.PublishAsset(context, "creator", channelId, assetSource, true);
    logger.log(1, `Asset published with id: ${publishId}`);

    // Step 3: Flag the asset with "hateful" using another actor
    let flag = "hateful" as Flag;
    await channelHelper.FlagAsset(context, "viewer", channelId, publishId, flag);
    logger.log(1, `Asset flagged as hateful with id: ${publishId}`);

    // Step 4: Query the asset
    let asset = await channelHelper.QueryAsset(context, "viewer", channelId, publishId);
    logger.log(1, `Asset Flags: ${JSON.stringify(asset
    )}`);

};

testFlaggingLogic().catch((error) => {
    logger.error(`Error in flagging logic test: ${error}`);
});