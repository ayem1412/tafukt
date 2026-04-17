# ⵜⴰⴼⵓⴽⵜ
[![tafukt](https://github.com/ayem1412/tafukt/blob/fd5d7aeba65b7e49e191e1ea67114a21272f3585/assets/tafukt.jpg "tafukt")](https://en.wikipedia.org/wiki/Tafukt)

Bittorrent client in rust

### Building

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

git clone git@github.com:ayem1412/tafukt.git
cd tafukt
cargo run
```

## Features

### Supported BEPs

- [x] [BEP-3: The BitTorrent Protocol Specification](https://www.bittorrent.org/beps/bep_0003.html)
- [ ] [BEP-5: DHT Protocol](https://www.bittorrent.org/beps/bep_0005.html)
- [ ] [BEP-7: IPv6 Tracker Extension](https://www.bittorrent.org/beps/bep_0007.html)
- [ ] [BEP-9: Extension for Peers to Send Metadata Files](https://www.bittorrent.org/beps/bep_0009.html)
- [ ] [BEP-10: Extension Protocol](https://www.bittorrent.org/beps/bep_0010.html)
- [ ] [BEP-11: Peer Exchange (PEX)](https://www.bittorrent.org/beps/bep_0011.html)
- [ ] [BEP-12: Multitracker Metadata Extension](https://www.bittorrent.org/beps/bep_0012.html)
- [ ] [BEP-14: Local service discovery](https://www.bittorrent.org/beps/bep_0014.html)
- [ ] [BEP-15: UDP Tracker Protocol](https://www.bittorrent.org/beps/bep_0015.html)
- [x] [BEP-20: Peer ID Conventions](https://www.bittorrent.org/beps/bep_0020.html)
- [x] [BEP-23: Tracker Returns Compact Peer Lists](https://www.bittorrent.org/beps/bep_0023.html)
- [ ] [BEP-27: Private Torrents](https://www.bittorrent.org/beps/bep_0027.html)
- [ ] [BEP-29: uTorrent Transport Protocol](https://www.bittorrent.org/beps/bep_0029.html)
- [ ] [BEP-32: IPv6 extension for DHT](https://www.bittorrent.org/beps/bep_0032.html)
- [ ] [BEP-47: Padding files and extended file attributes](https://www.bittorrent.org/beps/bep_0047.html)
- [ ] [BEP-53: Magnet URI extension - Select specific file indices for download](https://www.bittorrent.org/beps/bep_0053.html)

### Credits

Specifications:  
https://www.bittorrent.org/beps/bep_0000.html  
https://en.wikipedia.org/wiki/Torrent_file  

Thanks to these sources for the inspiration:  
https://github.com/denis-selimovic/bencode  
https://www.nayuki.io/res/bittorrent-bencode-format-tools/bencode.rs  
https://codeberg.org/benjamingeer/sayaca  
https://blog.jse.li/posts/torrent/  
