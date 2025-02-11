import Context from '../helpers/context';
import ChannelHelper from '../helpers/channel.helper';
import { logger } from '../utils/logger';
import { ReservedUsername } from '../types/OmniFlixChannel.types';


const addReservedUsernames = async () => {
    let context = new Context;
    await context.initialize();
    let channel_helper = new ChannelHelper;
    // Instantiate new channel contract
    await channel_helper.InstantiateChannelContract(context);
    // Default reserved username is 'reserved'
    // Query reserved usernames
    await channel_helper.QueryReservedUsernames(context);
    logger.log(1, 'Add 100 reserved usernames');
    // Generate an array of 24 reserved usernames using lowercase alphabets
    let reservedUsernames: ReservedUsername[] = [];
    for (let i = 0; i < 24; i++) {
        // Generate a random username of length 8 (can be adjusted)
        let reservedUsername: ReservedUsername = { username: '' };
        for (let j = 0; j < 8; j++) {
            // Get a random lowercase letter (ASCII 'a' to 'z')
            reservedUsername.username += String.fromCharCode(97 + Math.floor(Math.random() * 26));
        }
        reservedUsernames.push(reservedUsername);
    }
    console.log(reservedUsernames);
    await channel_helper.AddReservedUsernames(context, reservedUsernames);

    // Query reserved usernames
    await channel_helper.QueryReservedUsernames(context);

    // Readd the first usernmame with a valid address
    reservedUsernames[0].address = context.getTestUser('creator')?.address;
    logger.log(1, `Setting creators address as the first reserved username ${reservedUsernames[0].username}`);
    await channel_helper.AddReservedUsernames(context, [reservedUsernames[0]]);

    // Query reserved usernames
    await channel_helper.QueryReservedUsernames(context);

    // Create a new channel with reserved username
    await channel_helper.CreateChannel(context, "creator", reservedUsernames[0].username);
    logger.log(1, `Created a new channel with reserved username ${reservedUsernames[0].username}`);

    // Query reserved usernames
    await channel_helper.QueryReservedUsernames(context);

}


addReservedUsernames().then(() => {
    logger.info('Channel created successfully');
}).catch((error) => {
    logger.error(`Error creating channel: ${error}`);
});

