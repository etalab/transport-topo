# script needed by wdqs-updater
# we override it to give a custom "poll-delay" value (with the -d param)
# so the updater is run more often

cd /wdqs

# TODO env vars for entity namespaces, scheme and other settings
/wait-for-it.sh $WIKIBASE_HOST:80 -t 180 -- \
/wait-for-it.sh $WDQS_HOST:$WDQS_PORT -t 180 -- \
./runUpdate.sh -h http://$WDQS_HOST:$WDQS_PORT -- \
--wikibaseHost $WIKIBASE_HOST \
--wikibaseScheme $WIKIBASE_SCHEME \
--entityNamespaces $WDQS_ENTITY_NAMESPACES \
-d $WAITING_POLL_TIME_IN_S
