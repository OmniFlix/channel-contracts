
import { Field, Type } from 'protobufjs';
import { logger } from '../utils/logger';
import Context from './context';
import { MsgGrant } from 'cosmjs-types/cosmos/authz/v1beta1/tx'
import { GenericAuthorization, Grant } from 'cosmjs-types/cosmos/authz/v1beta1/authz';
import { EncodeObject } from '@cosmjs/proto-signing';

export default class AuthzHelper {
    giveAuthorization = async (context: Context, sender: string, grantee: string) => {
        const { client: senderClient, address: senderAddress } = context.getTestUser(sender);
        const { address: granteeAddress } = context.getTestUser(grantee);


        // grant: {
        //     authorization: {
        //       typeUrl: '/cosmos.authz.v1beta1.GenericAuthorization',
        //       value: GenericAuthorization.encode(
        //         GenericAuthorization.fromPartial({
        //           msg: typeURL,
        //         }),
        //       ).finish(),
        //     },
        //     expiration: expiration ? { seconds: expiration } : undefined,
        //   },
        let grant: Grant = {
            authorization: {
                typeUrl: '/cosmos.authz.v1beta1.GenericAuthorization',
                value: GenericAuthorization.encode(
                    GenericAuthorization.fromPartial({
                        msg: "/cosmwasm.wasm.v1.MsgExecuteContract",
                    }),
                ).finish(),
            },
            expiration: undefined
        }

        interface MsgGrantAuthorizationEncodeObject extends EncodeObject {
            readonly typeUrl: '/cosmos.authz.v1beta1.MsgGrant';
            readonly value: Partial<MsgGrant>;
        }

        let msgGrant = MsgGrant.fromPartial({
            granter: senderAddress,
            grantee: granteeAddress,
            grant: grant
        });

        let msgGrantAuthorizationExecute: MsgGrantAuthorizationEncodeObject = {
            typeUrl: '/cosmos.authz.v1beta1.MsgGrant',
            value: msgGrant
        }

        let result = await senderClient.signAndBroadcast(senderAddress, [msgGrantAuthorizationExecute], "auto");

        logger.info(`grant authorization from ${sender} to ${grantee} with result: ${result.transactionHash}`);







    }
}
