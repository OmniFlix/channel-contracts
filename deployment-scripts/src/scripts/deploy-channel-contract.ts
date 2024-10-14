import Context from '../helpers/context';
import ChannelHelper from '../helpers/channel.helper';
import { logger } from '../utils/logger';


const deployChannelContract = async () => {
    let context = new Context;
    await context.initialize();
    let channel_helper = new ChannelHelper;
    await channel_helper.InstantiateChannelContract(context);
}

deployChannelContract().then(() => {
    logger.log(1, 'Channel contract deployed successfully');
}).catch((e) => {
    logger.error(`Error deploying stream swap controller: ${e}`);
});

