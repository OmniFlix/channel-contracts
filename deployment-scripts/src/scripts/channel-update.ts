import Context from '../helpers/context';
import ChannelHelper from '../helpers/channel.helper';
import { logger } from '../utils/logger';


const channelUpdate = async () => {
    let context = new Context;
    await context.initialize();
    let channel_helper = new ChannelHelper;

    // Instantiate new channel contract
    await channel_helper.InstantiateChannelContract(context);

    let testingUserName = "creator"

    // Create channel
    let channelId = await channel_helper.CreateChannel(context, "creator", testingUserName);

    // Query channel metadata 
    await channel_helper.QueryChannelMetadata(context, "creator", channelId);

    // Update channel description
    let newDescription = "OmniFlix Channel Testing - Updated description";
    logger.log(1, `Updating channel with new description: ${newDescription}`);

    await channel_helper.UpdateChannelDetails(context, "creator", channelId, newDescription);

    // Query channel metadata again
    await channel_helper.QueryChannelMetadata(context, "creator", channelId);

    // Update channel name
    let newName = "Updatedname";
    logger.log(1, `Updating channel with new name: ${newName}`);

    await channel_helper.UpdateChannelDetails(context, "creator", channelId, undefined, newName);

    // Query channel metadata again
    await channel_helper.QueryChannelMetadata(context, "creator", channelId);

    // Update channel banner
    let newBanner = "https://www.omniflix.com/images/banner.jpg";
    logger.log(1, `Updating channel with new banner: ${newBanner}`);

    await channel_helper.UpdateChannelDetails(context, "creator", channelId, undefined, undefined, newBanner);

    // Query channel metadata again
    await channel_helper.QueryChannelMetadata(context, "creator", channelId);


    // Update channel profile picture
    let newProfilePicture = "https://www.omniflix.com/images/profile.jpg";
    logger.log(1, `Updating channel with new profile picture: ${newProfilePicture}`);

    await channel_helper.UpdateChannelDetails(context, "creator", channelId, undefined, undefined, undefined, newProfilePicture);

    // Query channel metadata again
    await channel_helper.QueryChannelMetadata(context, "creator", channelId);

    // Query channel details 
    await channel_helper.QueryChannelDetails(context, "creator", channelId);

    // Update channel collaborators
    let newCollaborators = [context.getTestUser("creator").address, context.getTestUser("admin").address];
    logger.log(1, `Updating channel with new collaborators: ${newCollaborators}`);

    await channel_helper.UpdateChannelDetails(context, "creator", channelId, undefined, undefined, undefined, undefined, newCollaborators);

    // Query channel details 
    await channel_helper.QueryChannelDetails(context, "creator", channelId);
}

channelUpdate()

