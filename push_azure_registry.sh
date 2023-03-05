read -p "Press any key to continue..." -n1 -s

docker image tag line-botx:latest eaacrglobal101.azurecr.io/line-botx:latest
az login

read -p "Press any key to continue... " -n1 -s

az acr login --name eaacrglobal101
docker push eaacrglobal101.azurecr.io/line-botx:latest
