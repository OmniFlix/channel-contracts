# OmniFlix Channel Contract

## Overview

The OmniFlix Channel Contract is a CosmWasm-based smart contract deployed on the OmniFlix Hub that enables creators to publish and manage video content in a decentralized manner. This contract serves as the foundation for OmniFlix's content ecosystem, supporting VOD (video-on-demand), live streams, and more.

## Features

### Channel Management
- **Channel Creation**: Create a personalized channel with a unique username, channel name, description, and visual assets
- **Channel Ownership**: Each channel is represented by an NFT (ONFT) that proves ownership
- **Channel Metadata**: Update channel details including name, description, profile picture, and banner


### Content Publishing
- **Asset Publishing**: Publish content in multiple formats - either as NFTs with ownership rights or as off-chain media hosted on decentralized storage like IPFS.
- **Asset Visibility**: Granular control over content visibility, allowing you to make specific assets public or private with a simple toggle.
- **Asset Management**: Comprehensive tools to update metadata or completely remove published content from your channel.

### Collaboration
- **Collaborator Management**: Add team members to your channel with specific roles (Moderator, Publisher) to help manage content and operations.
- **Revenue Sharing**: Sophisticated built-in mechanism for distributing revenue among collaborators based on customizable percentage shares.
- **Role-Based Permissions**: Different access levels ensure collaborators can only perform actions appropriate to their role, maintaining channel security. 

### Playlists
- **Playlist Creation**: Organize your content into themed playlists to enhance viewer experience and content discovery.
- **Playlist Management**: Easily add, remove, or refresh assets in playlists to keep your content organized and up-to-date.
- **Cross-Channel Playlists**: Create curated experiences by including content from other channels in your playlists, fostering community collaboration.

### Community Engagement
- **Channel Following**: Build your audience with a following system that allows users to stay updated with your latest content.
- **Content Flagging**: Community-driven moderation through a sophisticated flagging system that helps maintain content quality.
- **Creator Tipping**: Direct financial support mechanism allowing viewers to tip creators with native tokens, creating additional revenue streams.

### Administration
- **Reserved Usernames**: Advanced system for reserving and managing usernames, protecting brand identities and premium handles.
- **Contract Configuration**: Flexible configuration options for fees, administrative settings, and operational parameters.
- **Pause Mechanism**: Emergency safety feature to pause contract operations if needed, protecting both creators and users.

## Getting Started

### Prerequisites
- CosmWasm-compatible blockchain (OmniFlix Hub)
- Rust toolchain for development
- Access to OmniFlix Studio UI (recommended)

### Installation

```bash
git clone https://github.com/OmniFlix/omniflix-channel.git
cd omniflix-channel
cargo build
```

### Deployment

The contract can be deployed using standard CosmWasm deployment procedures:

```bash
# Example deployment command
omniflixhubd tx wasm store artifacts/omniflix_channel.wasm --from <your-key> --chain-id <chain-id> --gas auto --gas-adjustment 1.3 -y
```

### Initialization

Initialize the contract with the following parameters:

```json
{
  "protocol_admin": "<admin-address>",
  "fee_collector": "<fee-collector-address>",
  "channels_collection_id": "<collection-id>",
  "channels_collection_name": "OmniFlix Channels",
  "channels_collection_symbol": "OFXC",
  "channel_creation_fee": [{"denom": "uflix", "amount": "1000000"}],
  "accepted_tip_denoms": ["uflix"],
  "reserved_usernames": []
}
```

## Usage

### Creating a Channel

```bash
omniflixhubd tx wasm execute <contract-address> '{
  "channel_create": {
    "salt": "<random-binary>",
    "user_name": "mychannel",
    "channel_name": "My Channel",
    "description": "A channel for my content",
    "payment_address": "<payment-address>",
    "profile_picture": "https://example.com/profile.jpg",
    "banner_picture": "https://example.com/banner.jpg"
  }
}' --amount 1000000uflix --from <your-key>
```

### Publishing Content

```bash
omniflixhubd tx wasm execute <contract-address> '{
  "asset_publish": {
    "asset_source": {
      "off_chain": {
        "media_uri": "ipfs://Qm...",
        "name": "My Video",
        "description": "An awesome video"
      }
    },
    "salt": "<random-binary>",
    "channel_id": "<channel-id>",
    "playlist_name": "My Playlist",
    "is_visible": true
  }
}' --from <your-key>
```

### Creating a Playlist

```bash
omniflixhubd tx wasm execute <contract-address> '{
  "playlist_create": {
    "playlist_name": "My Playlist",
    "channel_id": "<channel-id>"
  }
}' --from <your-key>
```

### Adding Collaborators

```bash
omniflixhubd tx wasm execute <contract-address> '{
  "channel_add_collaborator": {
    "channel_id": "<channel-id>",
    "collaborator_address": "<collaborator-address>",
    "collaborator_details": {
      "role": "Moderator",
      "share": "0.2"
    }
  }
}' --from <your-key>
```

## Query Operations

### Channel Details

```bash
omniflixhubd query wasm contract-state smart <contract-address> '{
  "channel": {
    "channel_id": "<channel-id>"
  }
}'
```

### Channel Assets

```bash
omniflixhubd query wasm contract-state smart <contract-address> '{
  "assets": {
    "channel_id": "<channel-id>",
    "limit": 10
  }
}'
```

### Channel Playlists

```bash
omniflixhubd query wasm contract-state smart <contract-address> '{
  "playlists": {
    "channel_id": "<channel-id>",
    "limit": 10
  }
}'
```

## Username Reservation System

The contract includes a username reservation system that allows specific usernames to be reserved for particular addresses or marked as generally reserved. This system ensures that premium or brand-specific usernames can be protected.

- Usernames can be reserved with or without a specific address assignment
- Reserved usernames without an address assignment cannot be claimed by anyone
- Reserved usernames with an address assignment can only be claimed by that address

## Security Features

- **Ownership Verification**: All operations verify the sender is authorized
- **Pause Mechanism**: Contract can be paused in case of emergencies
- **Role-Based Access**: Different permissions for owners, collaborators, and admins

## License

This project is licensed under [LICENSE] - see the LICENSE file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Contact

For questions or support, please reach out to the OmniFlix team:
- Website: [https://omniflix.network](https://omniflix.network)
- Twitter: [@OmniFlixNetwork](https://twitter.com/OmniFlixNetwork)
- Discord: [OmniFlix Discord](https://discord.gg/omniflix)
