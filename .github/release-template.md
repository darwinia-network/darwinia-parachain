Darwinia Parachain
===

|      Network       |              Native Runtime              | Upgrade Priority |
| :----------------: | :--------------------------------------: | :--------------: |
| Darwinia Parachain | {{ darwinia_parachain_runtime_version }} |       LOW        |
|   Crab Parachain   |   {{ crab_parachain_runtime_version }}   |       LOW        |

## Resources

### Pre-built Binary
|  OS   |  Arch  | Glibc | LLVM  |                                                                                                                                       Download                                                                                                                                       |
| :---: | :----: | :---: | :---: | :----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------: |
| Linux | x86_64 | 2.27  |  4.0  | [tar.bz2](https://github.com/darwinia-network/darwinia-parachain/releases/download/{{ tag }}/darwinia-parachain-x86_64-linux-gnu.tar.bz2), [tar.zst](https://github.com/darwinia-network/darwinia-parachain/releases/download/{{ tag }}/darwinia-parachain-x86_64-linux-gnu.tar.zst) |

### Docker
```docker
docker pull quay.io/darwinia-network/darwinia-parachain:{{ tag }}
```

## Proposal Hashes
|      Network       |             Code Hash              |
| :----------------: | :--------------------------------: |
| Darwinia Parachain | {{ darwinia_parachain_code_hash }} |
|   Crab Parachain   |   {{ crab_parachain_code_hash }}   |
