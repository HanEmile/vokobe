# sync files
rsync -avz --delete ../emile.space/out/* root@nix1:/var/www/emile.space/

# fix perms
ssh root@nix1 chmod -R +r /var/www/emile.space/*
