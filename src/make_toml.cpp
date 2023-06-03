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
};

namespace general {
int cnt = 0;
InEvent res[Maxn];
} // namespace general

namespace experience {
int cnt = 0;
InEvent res[Maxn];
bool cmp(InEvent a, InEvent b) {
  if (a.description != b.description)
    return a.description < b.description;
  return a.datetime < b.datetime;
}
} // namespace experience

int main() {
  freopen("with.txt", "r", stdin);
  for (; scanf("%[^\n] ", str) == 1; ++general::cnt) {
    string on = str;
    replace(on.begin(), on.end(), ';', ',');
    general::res[general::cnt].with = on;
  }
  fclose(stdin);
  freopen("descriptions.txt", "r", stdin);
  for (int i = 0; scanf("%[^\n] ", str) && i < general::cnt; ++i) {
    general::res[i].description = str;
  }
  fclose(stdin);
  freopen("datetime.txt", "r", stdin);
  for (int i = 0; scanf("%[^\n] ", str) && i < general::cnt; ++i) {
    general::res[i].datetime = str;
  }
  fclose(stdin);
  freopen("paths.txt", "r", stdin);
  for (int i = 0; scanf("%[^\n] ", str) && i < general::cnt; ++i) {
    general::res[i].path = str;
  }
  fclose(stdin);
  sort(general::res, general::res + general::cnt);

  freopen("image/experience/with.txt", "r", stdin);
  for (; scanf("%[^\n] ", str) == 1; ++experience::cnt) {
    string on = str;
    replace(on.begin(), on.end(), ';', ',');
    experience::res[experience::cnt].with = on;
  }
  fclose(stdin);
  freopen("image/experience/descriptions.txt", "r", stdin);
  for (int i = 0; scanf("%[^\n] ", str) && i < experience::cnt; ++i) {
    experience::res[i].description = str;
  }
  fclose(stdin);
  freopen("image/experience/datetime.txt", "r", stdin);
  for (int i = 0; scanf("%[^\n] ", str) && i < experience::cnt; ++i) {
    experience::res[i].datetime = str;
  }
  fclose(stdin);
  freopen("image/experience/paths.txt", "r", stdin);
  for (int i = 0; scanf("%[^\n] ", str) && i < experience::cnt; ++i) {
    experience::res[i].path = str;
  }
  fclose(stdin);
  sort(experience::res, experience::res + experience::cnt, experience::cmp);

  freopen("events.toml", "w", stdout);
  for (int i = 0, j; i < general::cnt; ++i) {
    printf("[[event]]\n");
    printf("description = \"%s\"\n", general::res[i].description.data());
    printf("image = [\n");
    for (j = i; j < general::cnt; ++j) {
      if (general::res[i].description == general::res[j].description) {
        printf("{ path = \"%s\", date = %s", general::res[j].path.data(),
               general::res[j].datetime.data());
        if (general::res[j].with != "0")
          printf(", with = [%s]", general::res[j].with.data());
        printf(" },\n");
      } else {
        break;
      }
    }
    printf("]\n");
    i = j;
  }

  for (int i = 0, j; i < experience::cnt; ++i) {
    printf("[[experience]]\n");
    printf("description = \"%s\"\n", experience::res[i].description.data());
    printf("image = [\n");
    for (j = i; j < experience::cnt; ++j) {
      if (experience::res[i].description == experience::res[j].description) {
        printf("{ path = \"%s\", date = %s", experience::res[j].path.data(),
               experience::res[j].datetime.data());
        if (experience::res[j].with != "0")
          printf(", with = [%s]", experience::res[j].with.data());
        printf(" },\n");
      } else {
        break;
      }
    }
    printf("]\n");
    i = j;
  }
  return 0;
}