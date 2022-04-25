Darwinia Parachain
===

## [{{ tag }}]

> :warning: **We changed the token decimal in this release. In order to query the correct value from RPC please upgrade your node.**

|    Network     |            Native Runtime            | Upgrade Priority |
| :------------: | :----------------------------------: | :--------------: |
| Crab Parachain | {{ crab_parachain_runtime_version }} |      MEDIUM      |

## Resources

### Pre-built Binary
|  OS   |  Arch  | Glibc | LLVM  |                                                                                                                                      Download                                                                                                                                      |
| :---: | :----: | :---: | :---: | :--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------: |
| Linux | x86_64 | 2.23  |  4.0  | [tar.bz2](https://github.com/darwinia-network/darwinia-parachain/releases/download/{{ tag }}/darwinia-collator-x86_64-linux-gnu.tar.bz2), [tar.zst](https://github.com/darwinia-network/darwinia-parachain/releases/download/{{ tag }}/darwinia-collator-x86_64-linux-gnu.tar.zst) |

### Docker
#### Pull with the Git Tag
```docker
docker pull quay.io/darwinia-network/darwinia-collator:{{ tag }}
```
#### Pull with the Git Commit SHA
```docker
docker pull quay.io/darwinia-network/darwinia-collator:sha-{{ sha }}
```

## Proposal Hashes
|    Network     |             Proposal Hash             |
| :------------: | :-----------------------------------: |
| Crab Parachain | {{ crab_parachain_proposal_compact }} |
