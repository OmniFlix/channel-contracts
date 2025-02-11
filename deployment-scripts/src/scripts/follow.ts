import Context from '../helpers/context';
import ChannelHelper from '../helpers/channel.helper';
import { logger } from '../utils/logger';

const followChannel = async () => {
    let context = new Context;
    await context.initialize();

    let channelHelper = new ChannelHelper;

    // Instantiate new channel contract
    await channelHelper.InstantiateChannelContract(context);

    let testingUserName = "creator"

    // Create new channel
    let channelId = await channelHelper.CreateChannel(context, "creator", testingUserName);

    // Follow the channel using collector account
    try {
        await channelHelper.FollowChannel(context, "viewer", channelId);
        logger.log(1, "Successfully followed the channel");

        // Query followers count
        const followersCount = await channelHelper.QueryFollowersCount(context, channelId);
        logger.log(1, `Channel followers count: ${followersCount}`);

        // Query followers list
        const followers = await channelHelper.QueryFollowers(context, channelId);
        logger.log(1, `Channel followers: ${JSON.stringify(followers)}`);

        // Try to follow again (should fail)
        try {
            await channelHelper.FollowChannel(context, "viewer", channelId);
        } catch (error) {
            logger.log(1, `Expected error when following again: ${error}`);
        }

    } catch (error) {
        logger.log(3, `Error following channel: ${error}`);
        throw error;
    }
}

followChannel();
