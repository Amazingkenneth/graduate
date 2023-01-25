#include <algorithm>
#include <iostream>
#include <vector>
using namespace std;
const int Maxn = 1e4;
const int Maxlen = 1e3;
char str[Maxlen];
struct InEvent {
  string with, description, path, datetime;
  bool operator<(const InEvent cmp) const { return datetime < cmp.datetime; }
} res[Maxn];
int cnt = 0;
int main() {
  freopen("with.txt", "r", stdin);
  for (; scanf("%[^\n] ", str) == 1; ++cnt) {
    string on = str;
    replace(on.begin(), on.end(), ';', ',');
    res[cnt].with = on;
  }
  fclose(stdin);
  freopen("descriptions.txt", "r", stdin);
  for (int i = 0; scanf("%[^\n] ", str) && i < cnt; ++i) {
    res[i].description = str;
  }
  fclose(stdin);
  freopen("datetime.txt", "r", stdin);
  for (int i = 0; scanf("%[^\n] ", str) && i < cnt; ++i) {
    res[i].datetime = str;
  }
  fclose(stdin);
  freopen("paths.txt", "r", stdin);
  for (int i = 0; scanf("%[^\n] ", str) && i < cnt; ++i) {
    res[i].path = str;
  }
  fclose(stdin);
  freopen("events.toml", "w", stdout);
  sort(res, res + cnt);
  for (int i = 0, j; i < cnt; ++i) {
    printf("[[event]]\n");
    printf("description = \"%s\"\n", res[i].description.data());
    printf("image = [\n");
    for (j = i; j < cnt; ++j) {
      if (res[i].description == res[j].description) {
        printf("{ path = \"%s\", date = %s", res[j].path.data(),
               res[j].datetime.data());
        if (res[j].with != "0")
          printf(", with = [%s]", res[j].with.data());
        printf(" },\n");
      } else {
        i = j;
        break;
      }
    }
    printf("]\n");
  }
  return 0;
}
