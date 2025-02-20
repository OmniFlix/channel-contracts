import Context from '../helpers/context';
import ChannelHelper from '../helpers/channel.helper';
import { logger } from '../utils/logger';


const channelDelete = async () => {
    let context = new Context;
    await context.initialize();
    let channel_helper = new ChannelHelper;

    // Instantiate new channel contract
    await channel_helper.InstantiateChannelContract(context);

    let testingUserName = "creator" + Math.floor(Math.random() * 1000);

    // Create channel
    let channelId = await channel_helper.CreateChannel(context, "creator", testingUserName);

    // Query channel
    await channel_helper.QueryChannelDetails(context, "creator", channelId);

    // Delete channel
    await channel_helper.DeleteChannel(context, "creator", channelId);

    // Query channel
    await channel_helper.QueryChannelDetails(context, "creator", channelId).catch((error) => {
        logger.info("Channel not found");
    }
    );
}

channelDelete()

