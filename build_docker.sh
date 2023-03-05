#cur_dir=$(pwd)
#echo "remove chatgptproxy.linux"
#rm -rf chatgptproxy.linux

#echo "retry refresh new chatgptproxy from source code"

#cd /Users/s92612/Projects/Go/src/chatgptproxy
#rm -rf chatgptproxy.linux
#echo "build chat gpt"
#./build_linux.sh
#echo "build chat gpt proxy complete"

#cd $cur_dir

#cp /Users/s92612/Projects/Go/src/chatgptproxy/chatgptproxy.linux .
#echo "copy chatgptproxy.linux complete"

echo "building docker image ....."
docker rmi -f line-botx:latest
docker build -t line-botx .

docker images -a | grep none | awk '{ print $3; }' | xargs docker rmi
