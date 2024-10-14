import Context from '../helpers/context';
import ChannelHelper from '../helpers/channel.helper';


const setChannelConfig = async () => {
    let context = new Context;
    await context.initialize();
    let channel_helper = new ChannelHelper;

    // Instantiate new channel contract
    await channel_helper.InstantiateChannelContract(context);

    // Get channel contract config
    await channel_helper.QueryChannelConfig(context, "creator");

    // Update channel config
    await channel_helper.SetConfig(context, "admin", undefined, "400", "creator");

    // Get channel contract config
    await channel_helper.QueryChannelConfig(context, "creator");
}

setChannelConfig()

