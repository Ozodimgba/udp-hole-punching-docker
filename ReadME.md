### UDP Hole punching
This repo aims to simulate Hole punching; a NAT transversal techniques using docker containers

#### Components
- Signalling server: registers IP address of the external routers as well as port mapping, handles hole punching coordination
- Client: Handles P2P connectivity and peerdiscovery
- Docker-compose: simulates NAT barrier using 2 layers of network, one where the signaling server and the NATs containers have access to and the LANs
- IP-tables: I choose to use IP tables because they're least abstracted firewall rules I am conversant with
- Logging: Verbose ASCI logging

#### Replicate
First spin up the conatiners in the background with docker-compose
```bash
docker-compose up -d
```
Next accces Alice's and bob shell with
```bash
docker-compose exec alice bash

docker-compose exec bob bash
```
In their respective shells run the binaries with the known signaling server IP-ADDRESS:PORT
```bash
./target/debug/client alice 10.0.0.2:9090  

./target/debug/client bob 10.0.0.2:9090  
```
If successfull the client will be running on the alice and bob conatiners

I have added all the necessary commands to the client so you don't have to remember

To peform a hole punch coordination simply run on alice's client and vice versa
```bash
connect bob
```
P2P messages should now work, I send 10 bursts even when often times it works on the first I guess I have to delibratly track that.

Either way NAT transaversal works as the alice and bob can send P2P messages even with a Full cone NAT barrier
