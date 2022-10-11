package benchmark.repository;

import benchmark.model.Fortune;
import benchmark.model.World;
import io.javalin.util.ConcurrencyUtil;

import java.util.HashSet;
import java.util.List;
import java.util.Set;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.ThreadLocalRandom;

public interface DbService {

    int MIN_RANDOM_NUMBER = 1;
    int MAX_RANDOM_NUMBER_PLUS_ONE = 10001;
    int defaultFortuneId = 0;
    String defaultFortuneMessage = "Additional fortune added at request time.";

    ExecutorService executorService = ConcurrencyUtil.executorService("db-service");

    List<World> getWorld(int num);
    List<Fortune> getFortune();
    List<World> updateWorld(int num);

    default CompletableFuture<List<World>> getWorldAsync(int num){
        return CompletableFuture.supplyAsync(() -> getWorld(num), executorService);
    };
    default CompletableFuture<List<Fortune>> getFortuneAsync(){
        return CompletableFuture.supplyAsync(this::getFortune, executorService);
    };
    default CompletableFuture<List<World>> updateWorldAsync(int num){
        return CompletableFuture.supplyAsync(() -> updateWorld(num), executorService);
    };

    default int getRandomNumber() {
        return ThreadLocalRandom.current().nextInt(MIN_RANDOM_NUMBER, MAX_RANDOM_NUMBER_PLUS_ONE);
    }

    default Set<Integer> getRandomNumberSet(int num) {
        Set<Integer> set = new HashSet<>();
        while (set.size() < num)
            set.add(getRandomNumber());
        return set;
    }
}
