# Kubaefik-dns

Small DNS server used to redirect a kubernetes services/ingresses internal IP's.
It is intended to work with a Wireguard and Traefik to have "automatic" https with traefik withing the wireguard tunnel. 

This is a small specialized project not intended to be used without changes. 

# Configuration

Most of the configuration is happening in [config.rs](config.rs)

The IP used to redirect the services/ingresses of our Kubernetes is the address of our traefik web entrypoint.
I work with 2 kubernetes thus I have 2 traefik address and 2 kubernetes clients

The kubernetes clients are created in the [clients](kube.rs#L7) function
The traefik service names are defined in the [get_traefik_ingresses](kube.rs#L18) function


## Wireguard config

Example of what our wireguard config looks like

```conf
[Interface]
PrivateKey = X
Address = 10.192.0.3
; Address to this DNS running inside kubernetes
DNS = 10.40.11.210
; Failover DNS in case things don't work
DNS = 1.1.1.1
MTU = 1380

[Peer]
PublicKey = X
; Ip address range of service in Kubernetes. This depends on the configuration of you kubernetes, you probably want to change it. 
AllowedIPs = 10.40.0.0/16
; Address of the wireguard server
Endpoint = wg.test.com:51820
PersistentKeepalive = 25
```

## Traefik config

I simply use a middleware to restrict access only from within the server, and thus only allowing the Wireguard users

```yml
apiVersion: traefik.containo.us/v1alpha1
kind: Middleware
metadata:
  name: wireguard-ip-whitelist
  namespace: traefik
spec:
  ipWhiteList:
    sourceRange:
      - 10.2.0.0/8
```