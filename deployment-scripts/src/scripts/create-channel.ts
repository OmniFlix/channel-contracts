import Context from '../helpers/context';
import ChannelHelper from '../helpers/channel.helper';
import { logger } from '../utils/logger';


const createChannel = async () => {
    let context = new Context;
    await context.initialize();
    let channel_helper = new ChannelHelper;
    // Instantiate new channel contract
    await channel_helper.InstantiateChannelContract(context);

    let testing_user_name = "creator" + Math.floor(Math.random() * 1000);

    // Create new channel
    await channel_helper.CreateChannel(context, "creator", testing_user_name,)

}

createChannel().then(() => {
    logger.info('Channel created successfully');
}).catch((error) => {
    logger.error(`Error creating channel: ${error}`);
});

