export default async function (context, _myTimer) {
  context.log(new Date(), "General log message");
  context.log.error(new Date(), "Error log message; oh no, badness!");
  context.log.warn(new Date(), "Warning log message; be careful.");
  context.log.info(new Date(), "Informational log message");
  context.log.verbose(new Date(), "Chatty log message");
}
