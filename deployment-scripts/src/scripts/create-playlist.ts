import Context from '../helpers/context';
import ChannelHelper from '../helpers/channel.helper';
import OnftHelper from '../helpers/onft.helper';
import { logger } from '../utils/logger';

const createPlaylist = async () => {
    let context = new Context;
    await context.initialize();

    let channelHelper = new ChannelHelper;

    // Instantiate new channel contract
    await channelHelper.InstantiateChannelContract(context);

    let testingUserName = "creator" + Math.floor(Math.random() * 1000);

    // Create new channel
    let channelId = await channelHelper.CreateChannel(context, "creator", testingUserName);

    // Creating a playlist
    await channelHelper.CreatePlaylist(context, "creator", channelId, "PlaylistTest");

    // Query the playlist
    await channelHelper.QueryPlaylist(context, "creator", channelId, "PlaylistTest");

    // Refresh the playlist
    await channelHelper.RefreshPlaylist(context, "creator", channelId, "PlaylistTest");

    // Query the playlist again
    await channelHelper.QueryPlaylist(context, "creator", channelId, "PlaylistTest");

    // Delete the playlist
    await channelHelper.DeletePlaylist(context, "creator", channelId, "PlaylistTest");

    // Query the playlist again
    try {
        await channelHelper.QueryPlaylist(context, "creator", channelId, "PlaylistTest");
    } catch (error) {
        logger.log(1, `Expected error querying playlist: ${error}`);
    }
}

createPlaylist();
