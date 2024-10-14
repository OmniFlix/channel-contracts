import Context from './context'
import { Field, Type } from 'protobufjs'
import { logger } from '../utils/logger'
import { random } from 'lodash';
import { log } from 'console';


export default class OnftHelper {
    createOnftCollection = async (context: Context, sender: string, collection_id: string) => {
        let { client: senderClient, address: senderAddress } = context.getTestUser(sender);
        let createOnftCollectionMessage = {
            id: collection_id,
            symbol: "ONFT",
            name: "OmniFlix Channel Testing collection",
            description: "OmniFlix Channel Testing collection",
            preview_uri: "https://www.omniflix.com",
            schema: "https://www.omniflix.com/schema",
            sender: senderAddress,
            creation_fee: {
                amount: "1000000",
                denom: "uflix"
            },
            uri: "https://www.omniflix.com",
            uri_hash: "hash",
            data: "data",
            royalty_receivers: []
        }

        let createOnftCollectionTypeUrl = "/OmniFlix.onft.v1beta1.MsgCreateDenom"

        const CoinType = new Type('Coin')
            .add(new Field('denom', 1, 'string', 'required'))
            .add(new Field('amount', 2, 'string', 'required'));

        const WeightedAddress = new Type('WeightedAddress')
            .add(new Field('address', 1, 'string', 'required'))
            .add(new Field('weight', 2, 'string', 'required'));

        const MsgCreateDenom = new Type('MsgCreateDenom')
            .add(new Field('id', 1, 'string', 'required'))
            .add(new Field('symbol', 2, 'string', 'required'))
            .add(new Field('name', 3, 'string', 'required'))
            .add(new Field('description', 4, 'string', 'required'))
            .add(new Field('preview_uri', 5, 'string', 'required'))
            .add(new Field('schema', 6, 'string', 'required'))
            .add(new Field('sender', 7, 'string', 'required'))
            .add(new Field('creation_fee', 8, 'Coin', 'required'))
            .add(new Field('uri', 9, 'string', 'required'))
            .add(new Field('uri_hash', 10, 'string', 'required'))
            .add(new Field('data', 11, 'string', 'required'))
            .add(new Field('royalty_receivers', 12, 'WeightedAddress', 'repeated'));

        MsgCreateDenom.add(CoinType)
        MsgCreateDenom.add(WeightedAddress)

        senderClient.registry.register(createOnftCollectionTypeUrl, MsgCreateDenom);

        let EncodeObjectCreateOnftCollectionMessage = {
            typeUrl: createOnftCollectionTypeUrl,
            value: createOnftCollectionMessage
        }

        let res = await senderClient.signAndBroadcast(senderAddress, [EncodeObjectCreateOnftCollectionMessage], 2);
        logger.log(1, `Onft collection created with id: ${collection_id}`)
        logger.log(1, `Tx_Hash: ${res.transactionHash}\n`)
        return collection_id
    }

    mintOnft = async (context: Context, sender: string, recipient: string, collection_id: string, onft_id: string) => {
        let { client: senderClient, address: senderAddress } = context.getTestUser(sender);
        let recipientAddress = context.getTestUser(recipient).address
        let mintOnftMessage = {
            id: onft_id,
            denom_id: collection_id,
            metadata: {
                media_uri: "https://omniflix.tv/iv/66ee9752222d28ee22a403cb",
                name: "OmniFlix Channel Testing Asset",
                description: "OmniFlix Channel Testing Asset",
                preview_uri: "https://omniflix.tv/iv/66ee9752222d28ee22a403cb",
                uri_hash: "urihash"
            },
            data: "data",
            transferable: true,
            extensible: true,
            nsfw: false,
            royalty_share: "0",
            sender: senderAddress,
            recipient: recipientAddress
        }

        let mintOnftTypeUrl = "/OmniFlix.onft.v1beta1.MsgMintONFT"

        const Metadata = new Type('Metadata')
            .add(new Field('media_uri', 1, 'string', 'required'))
            .add(new Field('name', 2, 'string', 'required'))
            .add(new Field('description', 3, 'string', 'required'))
            .add(new Field('preview_uri', 4, 'string', 'required'))
            .add(new Field('uri_hash', 5, 'string', 'required'));

        const MsgMintOnft = new Type('MsgMintOnft')

        MsgMintOnft.add(new Field('id', 1, 'string', 'required'))
        MsgMintOnft.add(new Field('denom_id', 2, 'string', 'required'))
        MsgMintOnft.add(new Field('metadata', 3, 'Metadata', 'optional'))
        MsgMintOnft.add(new Field('data', 4, 'string', 'required'))
        MsgMintOnft.add(new Field('transferable', 5, 'bool', 'required'))
        MsgMintOnft.add(new Field('extensible', 6, 'bool', 'required'))
        MsgMintOnft.add(new Field('nsfw', 7, 'bool', 'required'))
        MsgMintOnft.add(new Field('royalty_share', 8, 'string', 'required'))
        MsgMintOnft.add(new Field('sender', 9, 'string', 'required'))
        MsgMintOnft.add(new Field('recipient', 10, 'string', 'required'))

        MsgMintOnft.add(Metadata)

        senderClient.registry.register(mintOnftTypeUrl, MsgMintOnft);

        let EncodeObjectMintOnftMessage = {
            typeUrl: mintOnftTypeUrl,
            value: mintOnftMessage
        }

        let res = await senderClient.signAndBroadcast(senderAddress, [EncodeObjectMintOnftMessage], 2);
        logger.log(1, `Onft minted with id: ${onft_id}`)
        logger.log(1, `Tx_Hash: ${res.transactionHash}\n`)
        return onft_id


    }
}