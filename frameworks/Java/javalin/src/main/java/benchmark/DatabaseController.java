package benchmark;

import benchmark.model.Fortune;
import benchmark.repository.DbService;
import io.javalin.http.Context;

import java.util.Collections;
import java.util.List;
import java.util.Map;

public class DatabaseController {

    private static final int MIN_QUERIES = 1;
    private static final int MAX_QUERIES = 500;

    private final DslJsonMapper jsonMapper;
    private final DbService dbService;

    public DatabaseController(DslJsonMapper jsonMapper, DbService dbService) {
        this.jsonMapper = jsonMapper;
        this.dbService = dbService;
    }

    public void handleSingleDbQuery(Context ctx) {
        ctx.future(()-> dbService.getWorldAsync(1)
                .thenAccept(worlds -> jsonMapper.writeJson(worlds.get(0), ctx)));
    }

    public void handleMultipleDbQueries(Context ctx) {
        int num = getBoundedRowNumber(ctx.queryParam("queries"));
        ctx.future(()-> dbService.getWorldAsync(num)
                .thenAccept(worlds -> jsonMapper.writeJson(worlds, ctx)));
    }

    public void handleFortunes(Context ctx) {
        ctx.future(()-> dbService.getFortuneAsync()
                .thenAccept(fortuneList -> {
                    Map<String, List<Fortune>> map = Collections.singletonMap("list", fortuneList);
                    ctx.render("fortune.jte", map).header("Content-Type", "text/html; charset=utf-8");
                }));
    }

    public void handleUpdates(Context ctx) {
        int num = getBoundedRowNumber(ctx.queryParam("queries"));
        ctx.future(()-> dbService.updateWorldAsync(num)
                .thenAccept(worlds -> jsonMapper.writeJson(worlds, ctx)));
    }

    private static int getBoundedRowNumber(String number) {
        int num;
        try {
            num = Integer.parseInt(number);
        } catch (NumberFormatException e) {
            num = MIN_QUERIES;
        }
        return Math.max(MIN_QUERIES, Math.min(num, MAX_QUERIES));
    }

}
