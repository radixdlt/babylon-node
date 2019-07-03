package org.radix.database;

import java.io.File;
import java.util.ArrayList;
import java.util.Collection;
import java.util.Collections;
import java.util.Comparator;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.concurrent.TimeUnit;

import org.bouncycastle.util.Arrays;
import org.radix.database.exceptions.DatabaseException;
import org.radix.logging.Logger;
import org.radix.logging.Logging;
import org.radix.modules.Modules;
import org.radix.modules.Service;
import org.radix.modules.exceptions.ModuleException;
import org.radix.modules.exceptions.ModuleStartException;
import org.radix.properties.RuntimeProperties;

import com.radixdlt.utils.RadixConstants;
import com.sleepycat.je.CacheMode;
import com.sleepycat.je.CheckpointConfig;
import com.sleepycat.je.Database;
import com.sleepycat.je.DatabaseConfig;
import com.sleepycat.je.DatabaseEntry;
import com.sleepycat.je.Durability;
import com.sleepycat.je.Environment;
import com.sleepycat.je.EnvironmentConfig;
import com.sleepycat.je.LockMode;
import com.sleepycat.je.OperationStatus;
import com.sleepycat.je.Transaction;

public final class DatabaseEnvironment extends Service
{
	private static final Logger log = Logging.getLogger();

	private class CheckpointerTask implements Runnable {
		private boolean interrupted = false;
		@Override
		public void run()
		{
			CheckpointConfig checkpointConfig = new CheckpointConfig();
			checkpointConfig.setForce(true);

			while (!interrupted) {
				try {
					long start = System.currentTimeMillis();

					while (DatabaseEnvironment.this.environment.cleanLogFile() && System.currentTimeMillis() - start < TimeUnit.MINUTES.toMillis(10)) {
						TimeUnit.SECONDS.sleep(10);
					}

					DatabaseEnvironment.this.environment.checkpoint(checkpointConfig);
					DatabaseEnvironment.this.environment.evictMemory();

					if (System.currentTimeMillis() - start < TimeUnit.MINUTES.toMillis(10)) {
						Thread.sleep(TimeUnit.MINUTES.toMillis(10) - (System.currentTimeMillis() - start));
					}
				} catch (InterruptedException ex) {
					Thread.currentThread().interrupt();
					interrupted = true;
				} catch (Exception ex) {
					log.error("Checkpointing of environment failed!", ex);
				}
			}
		}
	}

	private Database metaDatabase;

	private Environment						environment = null;
	private Thread 							checkpointThread = null;
	private Map<Class<?>, DatabaseStore> 	databases = new HashMap<>();

    public DatabaseEnvironment() { super(); }

	@Override
	public void start_impl() throws ModuleException
	{
		File dbhome = new File(Modules.get(RuntimeProperties.class).get("db.location", ".//RADIXDB"));
		dbhome.mkdir();

		System.setProperty("je.disable.java.adler32", "true");

		EnvironmentConfig environmentConfig = new EnvironmentConfig();
		environmentConfig.setTransactional(true);
		environmentConfig.setAllowCreate(true);
		environmentConfig.setLockTimeout(30, TimeUnit.SECONDS);
		environmentConfig.setDurability(Durability.COMMIT_NO_SYNC);
//		environmentConfig.setConfigParam(EnvironmentConfig.ENV_DUP_CONVERT_PRELOAD_ALL, "false");
		environmentConfig.setConfigParam(EnvironmentConfig.LOG_FILE_MAX, "100000000");
		environmentConfig.setConfigParam(EnvironmentConfig.LOG_FILE_CACHE_SIZE, "256");
		environmentConfig.setConfigParam(EnvironmentConfig.ENV_RUN_CHECKPOINTER, "false");
		environmentConfig.setConfigParam(EnvironmentConfig.ENV_RUN_CLEANER, "false");
		environmentConfig.setConfigParam(EnvironmentConfig.ENV_RUN_EVICTOR, "false");
		environmentConfig.setConfigParam(EnvironmentConfig.ENV_RUN_VERIFIER, "false");
//		environmentConfig.setConfigParam(EnvironmentConfig.NODE_MAX_ENTRIES, "256");
		environmentConfig.setConfigParam(EnvironmentConfig.TREE_MAX_EMBEDDED_LN, "0");

		long minCacheSize = Modules.get(RuntimeProperties.class).get("db.cache_size.min", Math.max(50000000, (long)(Runtime.getRuntime().maxMemory()*0.1)));
		long maxCacheSize = Modules.get(RuntimeProperties.class).get("db.cache_size.max", (long)(Runtime.getRuntime().maxMemory()*0.25));
		long cacheSize = Modules.get(RuntimeProperties.class).get("db.cache_size", (long)(Runtime.getRuntime().maxMemory()*0.125));
		cacheSize = Math.max(cacheSize, minCacheSize);
		cacheSize = Math.min(cacheSize, maxCacheSize);

		environmentConfig.setCacheSize(cacheSize);
		environmentConfig.setCacheMode(CacheMode.EVICT_LN);

		this.environment = new Environment(dbhome, environmentConfig);

		DatabaseConfig primaryConfig = new DatabaseConfig();
		primaryConfig.setAllowCreate(true);
		primaryConfig.setTransactional(true);

		try
		{
			this.metaDatabase = getEnvironment().openDatabase(null, "environment.meta_data", primaryConfig);
		}
        catch (Exception ex)
        {
        	throw new ModuleStartException(ex, this);
		}

		this.checkpointThread = new Thread(new CheckpointerTask());
		this.checkpointThread.setDaemon(true);
		this.checkpointThread.setName("Checkpointer");
		this.checkpointThread.start();
	}

	@Override
	public void stop_impl() throws ModuleException
	{
		Collection<DatabaseStore> allDatabases = new ArrayList<>(this.databases.values());
        for (DatabaseStore database : allDatabases)
        {
			try
        	{
				Modules.getInstance().stop(database);
			}
        	catch (ModuleException e)
			{
        		log.error("Failure stopping database "+database.getClass().getName(), e);
			}
        }

        try { flush(); } catch (DatabaseException dex)
        {
        	log.error("Flushing "+this.getClass().getName()+" on stop failed", dex);
		}

        this.metaDatabase.close();
		this.metaDatabase = null;

		this.checkpointThread.interrupt();
		try {
			this.checkpointThread.join();
		} catch (InterruptedException ex) {
			// Ignore and continue
			Thread.currentThread().interrupt();
		}

       	this.environment.close();
       	this.environment = null;
	}

	@Override
	public String getName()
	{
		return "Database Environment";
	}

	public Environment getEnvironment()
	{
		return this.environment;
	}

	public void flush() throws DatabaseException
	{
        for (DatabaseStore database : this.databases.values())
        	database.flush();
	}

    public void build() {
    	for (DatabaseStore database : this.getAll(true)) {
			try {
				database.build();
			} catch (DatabaseException dex) {
				log.error("Could not build database", dex);
			}
    	}
	}

	public void maintenence() {
        for (DatabaseStore database : this.databases.values()) {
			try {
				database.maintenence();
			} catch (DatabaseException dex) {
				log.error("Could not maintain database: " + database.toString(), dex);
			}
        }
	}

	public void register(DatabaseStore database) throws DatabaseException
	{
		if (this.databases.containsKey(database.getClass()) == false)
		{
			this.databases.put(database.getClass(), database);

/*			if (!ModuleManager.get(Universe.class).isProduction())
			{
	        	if (ModuleManager.get(SystemMetaData.class).opt("version", 0l) <= Radix.MAJOR_AGENT_VERSION  || clean)
	        	{
	        		clean = true;
	        		plugin.clean();
	                plugin.build();
	        	}
			}*/

			database.build();
		}
	}

	public boolean isRegistered(DatabaseStore database) {
		return this.databases.containsKey(database.getClass());
	}

	public void deregister(DatabaseStore database)
	{
		if (this.databases.containsKey(database.getClass()))
			this.databases.remove(database.getClass());
	}

	public List<DatabaseStore> getAll(boolean byPriority)
	{
		List<DatabaseStore> allDatabases = new ArrayList<>(this.databases.values());

		if (byPriority) {
			Collections.sort(allDatabases, Comparator.comparingInt(DatabaseStore::getBuildPriority));
		}

		return allDatabases;
	}

	public OperationStatus put(Transaction transaction, String resource, String key, byte[] value)
	{
		return this.put(transaction, resource, new DatabaseEntry(key.getBytes()), new DatabaseEntry(value));
	}

	public OperationStatus put(Transaction transaction, String resource, String key, DatabaseEntry value)
	{
		return this.put(transaction, resource, new DatabaseEntry(key.getBytes()), value);
	}

	public OperationStatus put(Transaction transaction, String resource, DatabaseEntry key, DatabaseEntry value)
	{
		if (resource == null || resource.length() == 0)
			throw new IllegalArgumentException("Resource can not be null or empty");

		if (key == null || key.getData() == null || key.getData().length == 0)
			throw new IllegalArgumentException("Key can not be null or empty");

		if (value == null || value.getData() == null || value.getData().length == 0)
			throw new IllegalArgumentException("Value can not be null or empty");

		// Create a key specific to the database //
		key.setData(Arrays.concatenate(resource.getBytes(RadixConstants.STANDARD_CHARSET), key.getData()));

		return this.metaDatabase.put(transaction, key, value);
	}

	public byte[] get(String resource, String key)
	{
		DatabaseEntry value = new DatabaseEntry();

		if (this.get(resource, new DatabaseEntry(key.getBytes()), value) == OperationStatus.SUCCESS)
			return value.getData();

		return null;
	}

	public OperationStatus get(String resource, String key, DatabaseEntry value)
	{
		return this.get(resource, new DatabaseEntry(key.getBytes()), value);
	}

	public OperationStatus get(String resource, DatabaseEntry key, DatabaseEntry value)
	{
		if (resource == null || resource.length() == 0)
			throw new IllegalArgumentException("Resource can not be null or empty");

		if (key == null || key.getData() == null || key.getData().length == 0)
			throw new IllegalArgumentException("Key can not be null or empty");

		if (value == null)
			throw new IllegalArgumentException("Value can not be null");

		// Create a key specific to the database //
		key.setData(Arrays.concatenate(resource.getBytes(RadixConstants.STANDARD_CHARSET), key.getData()));

		return this.metaDatabase.get(null, key, value, LockMode.READ_UNCOMMITTED);
	}
}
