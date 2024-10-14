import Context from '../helpers/context';
import ChannelHelper from '../helpers/channel.helper';
import { logger } from '../utils/logger';


const channelUpdate = async () => {
    let context = new Context;
    await context.initialize();
    let channel_helper = new ChannelHelper;

    // Instantiate new channel contract
    await channel_helper.InstantiateChannelContract(context);

    let testingUserName = "creator" + Math.floor(Math.random() * 1000);

    // Create channel
    let channelId = await channel_helper.CreateChannel(context, "creator", testingUserName);

    // Query channel
    await channel_helper.GetChannelDetails(context, "creator", channelId);

    // Update channel description
    let newDescription = "OmniFlix Channel Testing - Updated description";
    logger.log(1, `Updating channel with new description: ${newDescription}`);

    await channel_helper.UpdateChannelDetails(context, "creator", channelId, newDescription);

    // Query channel again
    await channel_helper.GetChannelDetails(context, "creator", channelId);
}

channelUpdate()

