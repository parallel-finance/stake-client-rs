define get_config_value
	$(shell sed -ne 's/^\s*$(1).*"\(.*\)"/\1/p' Config.toml | sed -n "$(2)p")
endef

PG_HOST                		:= $(strip $(call get_config_value,host,1))
PG_HOSTNAME            		:= $(word 1,$(subst :, ,$(PG_HOST)))
PG_PORT                		:= $(word 2,$(subst :, ,$(PG_HOST)))
PG_VERSION                  := 12
PG_USERNAME            		:= $(strip $(call get_config_value,username,1))
PG_PASSWORD            		:= $(strip $(call get_config_value,password,1))
PG_DATABASE            		:= $(strip $(call get_config_value,db,1))

.PHONY: run-db
run-db:
	if [ ! "$(shell docker container ls --all | grep staking_db)" ]; then \
		docker pull postgres:$(PG_VERSION) ;\
		docker tag  postgres:$(PG_VERSION) staking_db ;\
    fi ;\
    if [ ! "$(shell docker ps -q -f ancestor=staking_db)" ]; then \
		docker run -itd \
			--restart always \
			-e POSTGRES_USER=$(PG_USERNAME) \
			-e POSTGRES_PASSWORD=$(PG_PASSWORD) \
			-e POSTGRES_DB=$(PG_DATABASE) \
			-p $(PG_PORT):$(PG_PORT) \
			-v /var/folders/docker/postgresql:/var/lib/postgresql \
			staking_db ;\
		while true ;\
		do \
			if pg_isready -q -h $(PG_HOSTNAME) -p $(PG_PORT); then break; fi ;\
			sleep 1 ;\
		done ;\
		sleep 1 ;\
		cd db && diesel migration run --database-url \
			postgres://$(PG_USERNAME):$(PG_PASSWORD)@$(PG_HOSTNAME):$(PG_PORT)/$(PG_DATABASE) ;\
    else \
        echo "Staking db is already running!" ;\
    fi

.PHONY: rm-db
rm-db:
	if [ ! "$(shell docker ps -q -f ancestor=staking_db --all)" ]; then \
		echo "Staking db hasn't been created" ;\
    elif [ ! "$(shell docker ps -q -f ancestor=staking_db)" ]; then \
		docker rm $(shell docker ps -q -f ancestor=staking_db --all) ;\
	else \
		docker stop $(shell docker ps -q -f ancestor=staking_db) ;\
		docker rm $(shell docker ps -q -f ancestor=staking_db --all) ;\
	fi
