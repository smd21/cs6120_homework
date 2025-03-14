#include <iostream>
using namespace std;

int main(int argc, char **argv)
{
  int i = 15;
  int i2 = add_one(i);
  int b = 0;

  if (i2 > 0)
  {
    b = i2;
  }
  else
  {
    b = -i2;
  }
  cout << b << endl;
}

int add_one(int i)
{
  return i + 1;
}
