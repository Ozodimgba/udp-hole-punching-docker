### log my thoughts
```
gitpod /workspace/udp-hole-punching-docker (main) $ docker-compose exec alice-nat bash
tcpdump -i any -n udp port 9090 -v
root@0b658c7f66e5:/workspace# tcpdump -i any -n udp port 9090 -v
tcpdump: data link type LINUX_SLL2
tcpdump: listening on any, link-type LINUX_SLL2 (Linux cooked v2), snapshot length 262144 bytes
00:44:21.415406 eth0  In  IP (tos 0x0, ttl 64, id 41135, offset 0, flags [DF], proto UDP (17), length 36)
    192.168.1.3.46224 > 192.168.1.2.9090: UDP, length 8
00:44:21.415426 eth1  Out IP (tos 0x0, ttl 63, id 41135, offset 0, flags [DF], proto UDP (17), length 36)
    10.0.0.3.46224 > 10.0.0.2.9090: UDP, length 8
00:44:21.415533 eth1  In  IP (tos 0x0, ttl 64, id 53653, offset 0, flags [DF], proto UDP (17), length 51)
    10.0.0.2.9090 > 10.0.0.3.46224: UDP, length 23
00:44:21.415540 eth0  Out IP (tos 0x0, ttl 63, id 53653, offset 0, flags [DF], proto UDP (17), length 51)
    192.168.1.2.9090 > 192.168.1.3.46224: UDP, length 23

gitpod /workspace/udp-hole-punching-docker (main) $ docker-compose exec alice-nat bash
tcpdump -i any -n udp port 9090 -v
root@0b658c7f66e5:/workspace# tcpdump -i any -n udp port 9090 -v
tcpdump: data link type LINUX_SLL2
tcpdump: listening on any, link-type LINUX_SLL2 (Linux cooked v2), snapshot length 262144 bytes
00:44:21.415406 eth0  In  IP (tos 0x0, ttl 64, id 41135, offset 0, flags [DF], proto UDP (17), length 36)
    192.168.1.3.46224 > 192.168.1.2.9090: UDP, length 8
00:44:21.415426 eth1  Out IP (tos 0x0, ttl 63, id 41135, offset 0, flags [DF], proto UDP (17), length 36)
    10.0.0.3.46224 > 10.0.0.2.9090: UDP, length 8
00:44:21.415533 eth1  In  IP (tos 0x0, ttl 64, id 53653, offset 0, flags [DF], proto UDP (17), length 51)
    10.0.0.2.9090 > 10.0.0.3.46224: UDP, length 23
00:44:21.415540 eth0  Out IP (tos 0x0, ttl 63, id 53653, offset 0, flags [DF], proto UDP (17), length 51)
    192.168.1.2.9090 > 192.168.1.3.46224: UDP, length 23

gitpod /workspace/udp-hole-punching-docker (main) $ docker-compose exec signaling-server bash
root@32a97b963036:/workspace# tcpdump -i any -n udp port 9090 -v
tcpdump: data link type LINUX_SLL2
tcpdump: listening on any, link-type LINUX_SLL2 (Linux cooked v2), snapshot length 262144 bytes
00:44:21.415443 eth0  In  IP (tos 0x0, ttl 63, id 41135, offset 0, flags [DF], proto UDP (17), length 36)
    10.0.0.3.46224 > 10.0.0.2.9090: UDP, length 8
00:44:21.415519 eth0  Out IP (tos 0x0, ttl 64, id 53653, offset 0, flags [DF], proto UDP (17), length 51)
    10.0.0.2.9090 > 10.0.0.3.46224: UDP, length 23
```

```
Alice register's bobs ext address but Bob doesn't -> docke-compose exec alice bash # cargo run --bin client bob 192.168.2.2:9090

classic nat blocking, 

gitpod /workspace/udp-hole-punching-docker (main) $ docker-compose exec alice bash
root@66b70fddeec0:/workspace# tcpdump -i any -n udp port 9090 -v
tcpdump: data link type LINUX_SLL2
tcpdump: listening on any, link-type LINUX_SLL2 (Linux cooked v2), snapshot length 262144 bytes
02:29:53.160557 eth0  Out IP (tos 0x0, ttl 64, id 33642, offset 0, flags [DF], proto UDP (17), length 36)
    192.168.1.3.37455 > 192.168.1.2.9090: UDP, length 8
02:29:53.160746 eth0  In  IP (tos 0x0, ttl 63, id 31108, offset 0, flags [DF], proto UDP (17), length 51)
    192.168.1.2.9090 > 192.168.1.3.37455: UDP, length 23
02:29:53.160937 eth0  Out IP (tos 0x0, ttl 64, id 33643, offset 0, flags [DF], proto UDP (17), length 43)
    192.168.1.3.37455 > 192.168.1.2.9090: UDP, length 15
02:29:53.161078 eth0  In  IP (tos 0x0, ttl 63, id 31109, offset 0, flags [DF], proto UDP (17), length 67)
    192.168.1.2.9090 > 192.168.1.3.37455: UDP, length 39

gitpod /workspace/udp-hole-punching-docker (main) $ docker-compose exec bob bash  
tcpdump -i any -n "udp and not port 9090" -v
root@6a86f9509cb2:/workspace# tcpdump -i any -n "udp and not port 9090" -v
tcpdump: data link type LINUX_SLL2
tcpdump: listening on any, link-type LINUX_SLL2 (Linux cooked v2), snapshot length 262144 bytes

gitpod /workspace/udp-hole-punching-docker (main) $ docker-compose exec bob-nat bash
tcpdump -i any -n "udp and not port 9090" -v
root@9fb5c6014395:/workspace# tcpdump -i any -n "udp and not port 9090" -v
tcpdump: data link type LINUX_SLL2
tcpdump: listening on any, link-type LINUX_SLL2 (Linux cooked v2), snapshot length 262144 bytes

gitpod /workspace/udp-hole-punching-docker (main) $ docker-compose exec alice-nat bash
root@4e171a756f12:/workspace# tcpdump -i any -n "udp and not port 9090" -v
tcpdump: data link type LINUX_SLL2
tcpdump: listening on any, link-type LINUX_SLL2 (Linux cooked v2), snapshot length 262144 bytes

Alice communicates with server:9090

NO hole punch packets from Alice to Bob -> all other tcpdumps are empty

Bob never receives any packets to respond to

suspecting the iptables particularly the MASQUERADE -> could be causing symetric NAT.

Nat rules on Alice -> MASQUERADE  0    --  *      eth1    0.0.0.0/0            0.0.0.0/0

MASQUERADE - Changes source IP for outgoing packets
 - assign random port for each connection
 - only allows return traffice from the exact sme IP:port that was contacted
 looks like symetric mapping -> diff external ports for different dest

```

relaxed nat rules
```
08:14:40.300471 vethba47c25 P   IP 192.168.2.3.36502 > 10.0.0.3.34407: UDP, length 7
08:14:40.300471 br-f823e425d389 In  IP 192.168.2.3.36502 > 10.0.0.3.34407: UDP, length 7

packets enters bridge but brisdge is dropping them not routing
```

Hole punching works but i have a small bug I have to fix:

Bob can sent messages unless they send out punch packets...I dont think thats right.


hmmm hole punching success in consistent, tcpdump logs show:

# Alice sends from port 33831
192.168.1.3.33831 > 10.0.0.4.56907: UDP, length 7

# Bob responds from port 56907, but Alice expects it from a DIFFERENT port!  
192.168.2.3.56907 > 10.0.0.3.33831: UDP, length 7

but then bob NAT translates bob resposne to

10.0.0.4.7020 > 10.0.0.3.33831: UDP, length 7  

ip rules probably still symetric NAT

### Lunch break lol
Okay the rules work. should maybe go to nftables instead of nat?

relaxed the rules. Full cone nat should work

Masquerade is filiping the port on outbound.

this is you keep logs...SNAT fixed it

i should still monitor the tcpdump logs