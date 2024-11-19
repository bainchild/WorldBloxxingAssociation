rm -f certdata.txt
wget https://hg.mozilla.org/releases/mozilla-beta/raw-file/tip/security/nss/lib/ckfw/builtins/certdata.txt
openssl x509 -in cert.pem -pubin -outform DER -out cert.der
if [ "$#" -ge 1 ]
then
  name=$1
else
  name=$(openssl x509 -in cert.pem -nameopt RFC2253 -noout -subject | awk -F ',' '{print $3}' | sed 's/O=//')
fi
echo $name
nss-addbuiltin -n "$name" -t "C,C,C" -i cert.der >> certdata.txt
mk-ca-bundle -un cacert.pem
