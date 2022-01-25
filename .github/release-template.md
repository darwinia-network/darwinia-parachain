

## Upgrade Guide

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

| Network  |                Proposal Hash              |
| :------: | :---------------------------------------: |
| Darwinia | {{ darwinia_parachain_proposal_compact }} |
|   Crab   |   {{ crab_parachain_proposal_compact }}   |

