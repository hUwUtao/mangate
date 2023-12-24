const puppeteer = require("puppeteer");
const path = require("path");
const crypto = require("crypto");
const fs = require("fs");
// const { request } = require("http");

const express = require("express");
const app = express();

const regex = /https:\/\/(?:\w*.|)(?:hentaivn|htvncdn)\.\w*\//;
const rim = /\/[0-9]{10,}-([^\/]*\.\w*)\?imgmax/;

const hash = (url) => crypto.createHash("md5").update(url).digest("base64url");

(async () => {
  const browser = await puppeteer.launch({ headless: "new" });

  const page = await browser.newPage();

  await page.goto("https://hentaivn.red/p/ayame.php", {
    waitUntil: "load",
  });
  await page.setCookie({ name: "view1", value: "1" });

  app
    .use("/static", express.static("ifs"))
    .get("/api/fetch", async (req, res) => {
      const u = req.url.slice(req.url.indexOf("?") + 1);
      const hu = hash(u);

      if (fs.existsSync(`ifs/${hu}`)) {
        return res.send({
          st: "cached",
          cur: hu,
          ls: fs.readdirSync(`ifs/${hu}`),
        });
      }

      const page = await browser.newPage();
      await page.setRequestInterception(true);

      console.log(u);
      fs.mkdirSync(`ifs/${hu}`, { recursive: true });
      page
        .on("request", (request) => {
          if (
            ["a.realsrv.com", "js.smac-ad.com"].some((v) =>
              request.url().includes(v)
            )
          )
            request.abort();
          else request.continue();
        })
        .on("response", (response) => {
          if (
            response.url().includes("jpg?img") &&
            response.status() >= 200 &&
            response.status() < 300
          )
            response.buffer().then((file) => {
              console.log(response.url());
              const writeStream = fs.createWriteStream(
                path.resolve(
                  __dirname,
                  `ifs/${hu}/${rim.exec(response.url())[1]}`
                )
              );
              writeStream.write(file);
              writeStream.close()
            });
        });
      await page.goto(u, {
        waitUntil: "networkidle0",
      });
      await page.close();
      // await browser.close();
      res.send({ st: "fin", cur: hu, ls: fs.readdirSync(`ifs/${hu}`) });
    })
    .listen(8090);
})();
