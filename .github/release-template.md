Darwinia Parachain
===

## [5.1.0] - 2022-02-16

> :warning: **This release introduces a new host function. Please upgrade your node prior to the next runtime upgrade of Crab Parachain or Darwinia Parachain in order for your node to continue syncing.**

|      Network       | Native Runtime | Upgrade Priority |
| :----------------: | :------------: | :--------------: |
| Darwinia Parachain |       3        |       HIGH       |
|  Crab  Parachain   |       3        |       HIGH       |

## Resources

### Pre-built Binary
|  OS   |  Arch  | Glibc | LLVM  |                                                                 Download                                                                 |
| :---: | :----: | :---: | :---: | :--------------------------------------------------------------------------------------------------------------------------------------: |
| Linux | x86_64 | 2.23  |  4.0  | [tar.bz2](https://github.com/darwinia-network/darwinia-parachain/releases/download/{{ tag }}/darwinia-collator-x86_64-linux-gnu.tar.bz2) |

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
| Network  |                Proposal Hash                |
| :------: | :-----------------------------------------: |
| Darwinia | {{ z_darwinia_parachain_proposal_compact }} |
|   Crab   |   {{ z_crab_parachain_proposal_compact }}   |
