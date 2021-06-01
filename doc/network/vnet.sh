#!/bin/sh -u

ip link add br0 type bridge
ip link add dummy0 type dummy

sysctl net.ipv4.ip_forward=1

ip link set br0 up
ip link set dummy0 up
ip link set dummy0 master br0

for n in $(seq 0 1); do
    ip tuntap add dev "tap$n" mode tap
    ip link set "tap$n" master br0
    ip link set "tap$n" up
done

ip a add dev br0 172.16.0.1/24

iptables -t nat -A POSTROUTING -o eth0 -j MASQUERADE
iptables -t nat -A POSTROUTING -s 172.16.0.0/24 -j MASQUERADE
