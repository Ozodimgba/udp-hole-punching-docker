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